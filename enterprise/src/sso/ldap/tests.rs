//! LDAP sign-in against an in-process directory server that speaks real LDAP
//! (BER over TCP, optionally TLS or StartTLS), so the client's bind, search,
//! filter escaping, group handling and certificate verification all run.

use std::collections::HashMap;
use std::sync::Arc;

use bytes::BytesMut;
use ldap3::asn1::{
    ASNTag, Enumerated, Integer, OctetString, PL, Sequence, Set, StructureTag, Tag, TagClass,
    parse_tag, write,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use super::*;
use crate::sso::config::GroupRole;

const BASE: &str = "dc=corp,dc=example";

// ---------------------------------------------------------------------------
// The fake directory
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Entry {
    dn: String,
    attrs: Vec<(String, Vec<String>)>,
}

impl Entry {
    fn values(&self, name: &str) -> Vec<String> {
        self.attrs
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case(name))
            .flat_map(|(_, v)| v.clone())
            .collect()
    }
}

fn entry(dn: &str, attrs: &[(&str, &[&str])]) -> Entry {
    Entry {
        dn: dn.to_string(),
        attrs: attrs
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect(),
    }
}

struct Directory {
    entries: Vec<Entry>,
    /// Password by DN, for simple binds.
    passwords: HashMap<String, String>,
    /// Bind names recorded, to see which accounts a login used.
    binds: std::sync::Mutex<Vec<String>>,
    starttls: Option<TlsAcceptor>,
}

fn dn_eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

impl Directory {
    /// Is `dn` a member of `group`, directly or through nested groups?
    fn in_chain(&self, group: &Entry, dn: &str, depth: usize) -> bool {
        if depth > 8 {
            return false;
        }
        group.values("member").iter().any(|m| {
            dn_eq(m, dn)
                || self
                    .entries
                    .iter()
                    .find(|e| dn_eq(&e.dn, m))
                    .is_some_and(|sub| self.in_chain(sub, dn, depth + 1))
        })
    }

    fn matches(&self, e: &Entry, f: &StructureTag) -> bool {
        let children = |f: &StructureTag| match &f.payload {
            PL::C(c) => c.clone(),
            PL::P(_) => Vec::new(),
        };
        let text = |t: &StructureTag| match &t.payload {
            PL::P(p) => String::from_utf8_lossy(p).to_string(),
            PL::C(_) => String::new(),
        };
        match f.id {
            0 => children(f).iter().all(|c| self.matches(e, c)),
            1 => children(f).iter().any(|c| self.matches(e, c)),
            2 => !children(f).first().is_some_and(|c| self.matches(e, c)),
            3 => {
                let c = children(f);
                let (attr, val) = (text(&c[0]), text(&c[1]));
                e.values(&attr).iter().any(|v| v.eq_ignore_ascii_case(&val))
            }
            // Substring filter: any wildcard pattern on a present attribute matches.
            4 => !e.values(&text(&children(f)[0])).is_empty(),
            7 => !e.values(&text(f)).is_empty(),
            9 => {
                // extensibleMatch: matchingRule [1], type [2], matchValue [3]
                let c = children(f);
                let field = |id: u64| c.iter().find(|t| t.id == id).map(text).unwrap_or_default();
                let (rule, attr, value) = (field(1), field(2), field(3));
                rule == AD_IN_CHAIN && attr == "member" && self.in_chain(e, &value, 0)
            }
            _ => false,
        }
    }
}

fn ldap_result(op_id: u64, rc: i64, text: &str) -> Tag {
    Tag::Sequence(Sequence {
        id: op_id,
        class: TagClass::Application,
        inner: vec![
            Tag::Enumerated(Enumerated {
                inner: rc,
                ..Default::default()
            }),
            Tag::OctetString(OctetString::default()),
            Tag::OctetString(OctetString {
                inner: text.as_bytes().to_vec(),
                ..Default::default()
            }),
        ],
    })
}

fn message(id: i64, op: Tag) -> Vec<u8> {
    let msg = Tag::Sequence(Sequence {
        inner: vec![
            Tag::Integer(Integer {
                inner: id,
                ..Default::default()
            }),
            op,
        ],
        ..Default::default()
    });
    let mut buf = BytesMut::new();
    write::encode_into(&mut buf, msg.into_structure()).unwrap();
    buf.to_vec()
}

fn octets(s: &str) -> Tag {
    Tag::OctetString(OctetString {
        inner: s.as_bytes().to_vec(),
        ..Default::default()
    })
}

fn search_entry(e: &Entry, wanted: &[String]) -> Tag {
    let want_all = wanted.is_empty() || wanted.iter().any(|w| w == "*");
    let attrs: Vec<Tag> = e
        .attrs
        .iter()
        .filter(|(k, _)| want_all || wanted.iter().any(|w| w.eq_ignore_ascii_case(k)))
        .map(|(k, vals)| {
            Tag::Sequence(Sequence {
                inner: vec![
                    octets(k),
                    Tag::Set(Set {
                        inner: vals.iter().map(|v| octets(v)).collect(),
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            })
        })
        .collect();
    Tag::Sequence(Sequence {
        id: 4,
        class: TagClass::Application,
        inner: vec![
            octets(&e.dn),
            Tag::Sequence(Sequence {
                inner: attrs,
                ..Default::default()
            }),
        ],
    })
}

fn prim(t: &StructureTag) -> Vec<u8> {
    match &t.payload {
        PL::P(p) => p.clone(),
        PL::C(_) => Vec::new(),
    }
}

trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

/// One client connection. The stream is boxed so a StartTLS upgrade can swap
/// the transport and keep serving on the same code path.
async fn session(mut stream: Box<dyn Io>, dir: Arc<Directory>) {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let (consumed, tag) = loop {
            match parse_tag(&buf) {
                Ok((rest, tag)) => break (buf.len() - rest.len(), tag),
                Err(_) => {
                    let mut chunk = [0u8; 4096];
                    match stream.read(&mut chunk).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                }
            }
        };
        buf.drain(..consumed);

        let PL::C(parts) = tag.payload else { return };
        let id = i64::from_be_bytes({
            let p = prim(&parts[0]);
            let mut b = [0u8; 8];
            b[8 - p.len()..].copy_from_slice(&p);
            b
        });
        let op = &parts[1];
        let PL::C(fields) = &op.payload else {
            // Unbind is a primitive null.
            return;
        };

        match op.id {
            // BindRequest
            0 => {
                let name = String::from_utf8_lossy(&prim(&fields[1])).to_string();
                let pw = String::from_utf8_lossy(&prim(&fields[2])).to_string();
                dir.binds.lock().unwrap().push(name.clone());
                let ok = dir.passwords.get(&name).is_some_and(|p| *p == pw);
                let rc = if ok { 0 } else { 49 };
                let reply = message(
                    id,
                    ldap_result(1, rc, if ok { "" } else { "bad credentials" }),
                );
                if stream.write_all(&reply).await.is_err() {
                    return;
                }
            }
            // SearchRequest
            3 => {
                let base = String::from_utf8_lossy(&prim(&fields[0])).to_string();
                let filter = &fields[6];
                let wanted: Vec<String> = match &fields[7].payload {
                    PL::C(a) => a
                        .iter()
                        .map(|t| String::from_utf8_lossy(&prim(t)).to_string())
                        .collect(),
                    PL::P(_) => Vec::new(),
                };
                let mut out = Vec::new();
                for e in &dir.entries {
                    let under = e.dn.to_lowercase().ends_with(&base.to_lowercase());
                    if under && dir.matches(e, filter) {
                        out.extend(message(id, search_entry(e, &wanted)));
                    }
                }
                out.extend(message(id, ldap_result(5, 0, "")));
                if stream.write_all(&out).await.is_err() {
                    return;
                }
            }
            // ExtendedRequest: StartTLS
            23 => {
                let Some(acceptor) = dir.starttls.clone() else {
                    let reply = message(id, ldap_result(24, 2, "no StartTLS"));
                    let _ = stream.write_all(&reply).await;
                    continue;
                };
                let reply = message(id, ldap_result(24, 0, ""));
                if stream.write_all(&reply).await.is_err() {
                    return;
                }
                if let Ok(tls) = acceptor.accept(stream).await {
                    Box::pin(session(Box::new(tls), dir)).await;
                }
                return;
            }
            _ => return,
        }
    }
}

struct Server {
    port: u16,
    dir: Arc<Directory>,
}

async fn start(dir: Directory, tls: Option<TlsAcceptor>) -> Server {
    let dir = Arc::new(dir);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let d = dir.clone();
    tokio::spawn(async move {
        loop {
            let Ok((sock, _)) = listener.accept().await else {
                return;
            };
            let d = d.clone();
            let tls = tls.clone();
            tokio::spawn(async move {
                match tls {
                    Some(acceptor) => {
                        if let Ok(s) = acceptor.accept(sock).await {
                            session(Box::new(s), d).await;
                        }
                    }
                    None => session(Box::new(sock), d).await,
                }
            });
        }
    });
    Server { port, dir }
}

// ---------------------------------------------------------------------------
// Test PKI
// ---------------------------------------------------------------------------

struct Pki {
    ca_pem: String,
    acceptor: TlsAcceptor,
}

fn pki(server_names: &[&str]) -> Pki {
    use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, KeyPair};
    let ca_key = KeyPair::generate().unwrap();
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(DnType::CommonName, "Test CA");
    let ca = ca_params.self_signed(&ca_key).unwrap();

    let key = KeyPair::generate().unwrap();
    let params = CertificateParams::new(
        server_names
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let cert = params.signed_by(&key, &ca, &ca_key).unwrap();

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.der().clone()],
            rustls::pki_types::PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
        )
        .unwrap();
    Pki {
        ca_pem: ca.pem(),
        acceptor: TlsAcceptor::from(Arc::new(config)),
    }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn corp_directory() -> Directory {
    let alice = "CN=Alice Example,OU=People,dc=corp,dc=example";
    let bob = "CN=Bob Example,OU=People,dc=corp,dc=example";
    let ops = "CN=SCADA Operators,OU=Groups,dc=corp,dc=example";
    let admins = "CN=SCADA Admins,OU=Groups,dc=corp,dc=example";
    let plant = "CN=Plant East,OU=Groups,dc=corp,dc=example";
    let svc = "CN=svc-voltrus,OU=Service,dc=corp,dc=example";
    Directory {
        entries: vec![
            entry(
                alice,
                &[
                    ("sAMAccountName", &["Alice"]),
                    ("mail", &["alice@corp.example"]),
                    ("displayName", &["Alice Example"]),
                    ("memberOf", &[ops]),
                ],
            ),
            entry(
                bob,
                &[
                    ("sAMAccountName", &["bob"]),
                    ("mail", &["bob@corp.example"]),
                ],
            ),
            // Alice is in Operators directly; Operators is nested in Admins (via Plant East).
            entry(ops, &[("member", &[alice])]),
            entry(plant, &[("member", &[ops])]),
            entry(admins, &[("member", &[plant])]),
            entry(svc, &[("sAMAccountName", &["svc-voltrus"])]),
        ],
        passwords: HashMap::from([
            (alice.to_string(), "alice-pw".to_string()),
            (bob.to_string(), "bob-pw".to_string()),
            (svc.to_string(), "svc-pw".to_string()),
            ("alice@corp.example".to_string(), "alice-pw".to_string()),
        ]),
        binds: Default::default(),
        starttls: None,
    }
}

fn config(url: &str) -> LdapConfig {
    LdapConfig {
        enabled: true,
        url: url.to_string(),
        bind_dn: "CN=svc-voltrus,OU=Service,dc=corp,dc=example".into(),
        bind_password: "svc-pw".into(),
        base_dn: BASE.into(),
        user_filter: "(sAMAccountName={username})".into(),
        allow_plain_ldap: true,
        timeout_secs: 5,
        ..LdapConfig::default()
    }
}

async fn plain(dir: Directory) -> (Server, String) {
    let s = start(dir, None).await;
    let url = format!("ldap://127.0.0.1:{}", s.port);
    (s, url)
}

fn provider(cfg: LdapConfig) -> Arc<LdapProvider> {
    LdapProvider::new(cfg).unwrap()
}

// ---------------------------------------------------------------------------
// Authentication
// ---------------------------------------------------------------------------

#[tokio::test]
async fn valid_user_signs_in_with_profile_and_username_from_the_directory() {
    let (_srv, url) = plain(corp_directory()).await;
    let p = provider(LdapConfig {
        username_attribute: "sAMAccountName".into(),
        ..config(&url)
    });
    let u = p.authenticate("alice", "alice-pw").await.unwrap();
    assert_eq!(u.dn, "CN=Alice Example,OU=People,dc=corp,dc=example");
    assert_eq!(u.username, "alice", "directory attribute is lowercased");
    assert_eq!(u.email, "alice@corp.example");
    assert_eq!(u.display_name, "Alice Example");
}

#[tokio::test]
async fn wrong_password_and_unknown_user_look_the_same() {
    let (_srv, url) = plain(corp_directory()).await;
    let p = provider(config(&url));
    assert!(matches!(
        p.authenticate("alice", "nope").await,
        Err(LdapLoginError::InvalidCredentials)
    ));
    assert!(matches!(
        p.authenticate("nobody", "alice-pw").await,
        Err(LdapLoginError::InvalidCredentials)
    ));
}

#[tokio::test]
async fn the_typed_password_is_verified_by_binding_as_the_user() {
    let (srv, url) = plain(corp_directory()).await;
    let p = provider(config(&url));
    p.authenticate("bob", "bob-pw").await.unwrap();
    let binds = srv.dir.binds.lock().unwrap().clone();
    assert_eq!(
        binds,
        vec![
            "CN=svc-voltrus,OU=Service,dc=corp,dc=example".to_string(),
            "CN=Bob Example,OU=People,dc=corp,dc=example".to_string()
        ]
    );
}

#[tokio::test]
async fn empty_credentials_never_reach_the_directory() {
    let (srv, url) = plain(corp_directory()).await;
    let p = provider(config(&url));
    for (u, pw) in [
        ("alice", ""),
        ("", "alice-pw"),
        ("   ", "x"),
        ("a\nb", "x"),
        ("alice", "x\0y"),
    ] {
        assert!(
            matches!(
                p.authenticate(u, pw).await,
                Err(LdapLoginError::InvalidCredentials)
            ),
            "{u:?}"
        );
    }
    assert!(srv.dir.binds.lock().unwrap().is_empty());
}

#[tokio::test]
async fn filter_metacharacters_in_the_username_are_escaped() {
    // With a single user in the directory, an unescaped `*` would match that
    // user, and the typed password would then log in as them.
    let mut dir = corp_directory();
    dir.entries
        .retain(|e| e.dn.starts_with("CN=Alice") || e.dn.starts_with("CN=svc"));
    let (_srv, url) = plain(dir).await;
    let p = provider(config(&url));
    for evil in ["*", "al*", "alice)(sAMAccountName=*", "*)(objectClass=*"] {
        assert!(
            matches!(
                p.authenticate(evil, "alice-pw").await,
                Err(LdapLoginError::InvalidCredentials)
            ),
            "{evil}"
        );
    }
    assert!(p.authenticate("alice", "alice-pw").await.is_ok());
}

#[tokio::test]
async fn a_filter_matching_several_entries_refuses_the_login() {
    let mut dir = corp_directory();
    let mut twin = dir.entries[0].clone();
    twin.dn = "CN=Alice Twin,OU=People,dc=corp,dc=example".into();
    dir.entries.push(twin);
    let (_srv, url) = plain(dir).await;
    let p = provider(config(&url));
    assert!(matches!(
        p.authenticate("alice", "alice-pw").await,
        Err(LdapLoginError::InvalidCredentials)
    ));
}

#[tokio::test]
async fn service_account_failure_is_a_directory_error_not_a_bad_password() {
    let (_srv, url) = plain(corp_directory()).await;
    let p = provider(LdapConfig {
        bind_password: "wrong".into(),
        ..config(&url)
    });
    assert!(matches!(
        p.authenticate("alice", "alice-pw").await,
        Err(LdapLoginError::Directory(_))
    ));
}

#[tokio::test]
async fn unreachable_directory_is_a_directory_error() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let p = provider(config(&format!("ldap://127.0.0.1:{port}")));
    assert!(matches!(
        p.authenticate("alice", "alice-pw").await,
        Err(LdapLoginError::Directory(_))
    ));
}

#[tokio::test]
async fn direct_bind_mode_binds_as_the_formatted_name_then_reads_the_entry() {
    let (srv, url) = plain(corp_directory()).await;
    let p = provider(LdapConfig {
        bind_dn: String::new(),
        bind_password: String::new(),
        bind_dn_format: "{username}@corp.example".into(),
        ..config(&url)
    });
    // The fake knows this UPN-style name as a bind identity only for Alice.
    let u = p.authenticate("alice", "alice-pw").await.unwrap();
    assert_eq!(u.email, "alice@corp.example");
    assert_eq!(srv.dir.binds.lock().unwrap()[0], "alice@corp.example");
    assert!(matches!(
        p.authenticate("alice", "bad").await,
        Err(LdapLoginError::InvalidCredentials)
    ));
}

// ---------------------------------------------------------------------------
// Groups and roles
// ---------------------------------------------------------------------------

fn role_mapping() -> RoleMapping {
    RoleMapping {
        default_role: "viewer".into(),
        groups: vec![
            GroupRole {
                group: "CN=SCADA Operators,OU=Groups,DC=corp,DC=example".into(),
                role: "operator".into(),
            },
            GroupRole {
                group: "SCADA Admins".into(),
                role: "admin".into(),
            },
        ],
    }
}

#[tokio::test]
async fn member_of_gives_direct_groups_as_dn_and_cn() {
    let (_srv, url) = plain(corp_directory()).await;
    let cfg = LdapConfig {
        role_mapping: role_mapping(),
        ..config(&url)
    };
    let p = provider(cfg.clone());
    let u = p.authenticate("alice", "alice-pw").await.unwrap();
    assert!(
        u.groups
            .contains(&"CN=SCADA Operators,OU=Groups,dc=corp,dc=example".to_string())
    );
    assert!(u.groups.contains(&"SCADA Operators".to_string()));
    // Direct membership only: nested admin group is not visible through memberOf.
    assert_eq!(
        cfg.role_mapping.resolve(&u.groups).as_deref(),
        Some("operator")
    );
    // A user with no groups gets the default role.
    let bob = p.authenticate("bob", "bob-pw").await.unwrap();
    assert_eq!(
        cfg.role_mapping.resolve(&bob.groups).as_deref(),
        Some("viewer")
    );
}

#[tokio::test]
async fn nested_groups_are_resolved_through_the_directory() {
    let (_srv, url) = plain(corp_directory()).await;
    let cfg = LdapConfig {
        nested_groups: true,
        role_mapping: role_mapping(),
        ..config(&url)
    };
    let p = provider(cfg.clone());
    let u = p.authenticate("alice", "alice-pw").await.unwrap();
    // Operators -> Plant East -> Admins: the admin role arrives through two levels of nesting.
    assert!(
        u.groups.contains(&"SCADA Admins".to_string()),
        "{:?}",
        u.groups
    );
    assert!(u.groups.contains(&"Plant East".to_string()));
    assert_eq!(
        cfg.role_mapping.resolve(&u.groups).as_deref(),
        Some("admin")
    );
    // Bob is in no group at all, nested or not.
    let bob = p.authenticate("bob", "bob-pw").await.unwrap();
    assert!(bob.groups.is_empty());
}

#[tokio::test]
async fn custom_group_filter_is_escaped_and_applied() {
    let (_srv, url) = plain(corp_directory()).await;
    let p = provider(LdapConfig {
        member_of_attribute: String::new(),
        group_filter: "(member={user_dn})".into(),
        group_base_dn: "OU=Groups,dc=corp,dc=example".into(),
        ..config(&url)
    });
    let u = p.authenticate("alice", "alice-pw").await.unwrap();
    assert!(u.groups.contains(&"SCADA Operators".to_string()));
    assert!(
        !u.groups.contains(&"Plant East".to_string()),
        "non-nested filter finds direct groups only"
    );
}

#[test]
fn common_name_handles_escaped_commas() {
    assert_eq!(
        common_name(r"CN=Plant\, East,OU=Groups,DC=x").as_deref(),
        Some("Plant, East")
    );
    assert_eq!(common_name("OU=Groups,DC=x"), None);
}

// ---------------------------------------------------------------------------
// Transport security
// ---------------------------------------------------------------------------

#[test]
fn cleartext_ldap_needs_an_explicit_opt_in() {
    let mut cfg = config("ldap://dc.example.com");
    cfg.allow_plain_ldap = false;
    assert!(LdapProvider::new(cfg.clone()).is_err());
    cfg.starttls = true;
    // StartTLS needs a CA; the system store may be empty in CI, so give one.
    cfg.ca_cert = pki(&["dc.example.com"]).ca_pem;
    assert!(LdapProvider::new(cfg).is_ok());
}

#[tokio::test]
async fn ldaps_with_the_right_ca_succeeds() {
    let pki = pki(&["localhost"]);
    let srv = start(corp_directory(), Some(pki.acceptor.clone())).await;
    let p = provider(LdapConfig {
        allow_plain_ldap: false,
        ca_cert: pki.ca_pem,
        ..config(&format!("ldaps://localhost:{}", srv.port))
    });
    assert!(p.authenticate("alice", "alice-pw").await.is_ok());
}

#[tokio::test]
async fn ldaps_with_an_untrusted_ca_fails_closed() {
    let server_pki = pki(&["localhost"]);
    let other_ca = pki(&["localhost"]).ca_pem;
    let srv = start(corp_directory(), Some(server_pki.acceptor)).await;
    let p = provider(LdapConfig {
        allow_plain_ldap: false,
        ca_cert: other_ca,
        ..config(&format!("ldaps://localhost:{}", srv.port))
    });
    assert!(matches!(
        p.authenticate("alice", "alice-pw").await,
        Err(LdapLoginError::Directory(_))
    ));
    assert!(
        srv.dir.binds.lock().unwrap().is_empty(),
        "no credentials may be sent over an unverified channel"
    );
}

#[tokio::test]
async fn ldaps_checks_the_server_name() {
    let pki = pki(&["dc.example.com"]);
    let srv = start(corp_directory(), Some(pki.acceptor.clone())).await;
    let p = provider(LdapConfig {
        allow_plain_ldap: false,
        ca_cert: pki.ca_pem,
        // The certificate is for dc.example.com, we connect to localhost.
        ..config(&format!("ldaps://localhost:{}", srv.port))
    });
    assert!(matches!(
        p.authenticate("alice", "alice-pw").await,
        Err(LdapLoginError::Directory(_))
    ));
    assert!(srv.dir.binds.lock().unwrap().is_empty());
}

#[tokio::test]
async fn starttls_upgrades_before_any_bind_and_verifies_the_certificate() {
    let pki = pki(&["localhost"]);
    let mut dir = corp_directory();
    dir.starttls = Some(pki.acceptor.clone());
    let srv = start(dir, None).await;
    let url = format!("ldap://localhost:{}", srv.port);
    let p = provider(LdapConfig {
        allow_plain_ldap: false,
        starttls: true,
        ca_cert: pki.ca_pem,
        ..config(&url)
    });
    assert!(p.authenticate("alice", "alice-pw").await.is_ok());

    let wrong = provider(LdapConfig {
        allow_plain_ldap: false,
        starttls: true,
        ca_cert: self::pki(&["localhost"]).ca_pem,
        ..config(&url)
    });
    let before = srv.dir.binds.lock().unwrap().len();
    assert!(matches!(
        wrong.authenticate("alice", "alice-pw").await,
        Err(LdapLoginError::Directory(_))
    ));
    assert_eq!(
        srv.dir.binds.lock().unwrap().len(),
        before,
        "nothing is bound after a failed upgrade"
    );
}

#[test]
fn invalid_configuration_is_rejected_with_a_reason() {
    let base = config("ldap://dc.example.com");
    let cases: Vec<(LdapConfig, &str)> = vec![
        (
            LdapConfig {
                bind_dn: String::new(),
                ..base.clone()
            },
            "bind_dn",
        ),
        (
            LdapConfig {
                user_filter: "(uid=bob)".into(),
                ..base.clone()
            },
            "{username}",
        ),
        (
            LdapConfig {
                base_dn: String::new(),
                ..base.clone()
            },
            "base_dn",
        ),
        (
            LdapConfig {
                url: "http://x".into(),
                ..base.clone()
            },
            "scheme",
        ),
        (
            LdapConfig {
                url: "ldaps://x".into(),
                starttls: true,
                ..base.clone()
            },
            "starttls",
        ),
    ];
    for (cfg, wanted) in cases {
        let err = LdapProvider::new(cfg)
            .err()
            .unwrap_or_else(|| panic!("accepted config missing {wanted}"));
        assert!(err.contains(wanted), "wanted {wanted}, got {err}");
    }
}
