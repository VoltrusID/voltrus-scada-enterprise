//! SAML response validation against documents signed by openssl and libxml2
//! (see `fixtures/generate.sh`), plus string-level tampering of those documents.
//!
//! The fixtures are valid between 2029-12-31T23:55:00Z and 2030-01-01T00:05:00Z;
//! tests run at 2030-01-01T00:00:30Z unless they move the clock.

use super::*;

const SP: &str = "https://scada.example.com/api/v1/auth/saml/metadata";
const ACS: &str = "https://scada.example.com/api/v1/auth/saml/acs";
const IDP: &str = "https://idp.example.com/metadata";

const SIGNED_ASSERTION: &str = include_str!("fixtures/signed_assertion.xml");
const SIGNED_RESPONSE: &str = include_str!("fixtures/signed_response.xml");
const SIGNED_BOTH: &str = include_str!("fixtures/signed_both.xml");
const UNSIGNED: &str = include_str!("fixtures/unsigned.xml");
const IDP_PEM: &str = include_str!("fixtures/idp.pem");
const OTHER_PEM: &str = include_str!("fixtures/other.pem");

fn now() -> i64 {
    parse_time("2030-01-01T00:00:30Z").unwrap()
}

fn keys(pem: &str) -> Vec<RsaPublicKey> {
    vec![xmldsig::rsa_key_from_certificate(pem).unwrap()]
}

fn validate_at(
    xml: &str,
    pem: &str,
    at: i64,
    opts: VerifyOptions,
) -> Result<ValidatedAssertion, SamlError> {
    validate_response(
        xml,
        &ValidationContext {
            sp_entity_id: SP,
            acs_url: ACS,
            idp_entity_id: IDP,
            idp_keys: &keys(pem),
            now: at,
            skew: 60,
            opts,
        },
    )
}

fn validate(xml: &str) -> Result<ValidatedAssertion, SamlError> {
    validate_at(xml, IDP_PEM, now(), VerifyOptions::default())
}

fn refused(xml: &str, expect: &str) {
    match validate(xml) {
        Ok(_) => panic!("accepted a document that must be refused ({expect})"),
        Err(e) => assert!(e.0.contains(expect), "wanted '{expect}', got '{}'", e.0),
    }
}

// --- documents that must verify -------------------------------------------------

#[test]
fn signed_assertion_verifies_and_yields_its_claims() {
    let a = validate(SIGNED_ASSERTION).unwrap();
    assert_eq!(a.name_id, "alice@example.com");
    assert_eq!(a.in_response_to.as_deref(), Some("_req1"));
    assert_eq!(a.session_index.as_deref(), Some("_session1"));
    assert_eq!(a.attributes["groups"], vec!["scada-admins", "operators"]);
    // Attributes are reachable by FriendlyName too.
    assert_eq!(a.attributes["displayName"], vec!["Alice Example"]);
}

#[test]
fn signed_response_verifies() {
    assert_eq!(
        validate(SIGNED_RESPONSE).unwrap().name_id,
        "alice@example.com"
    );
}

#[test]
fn response_and_assertion_both_signed_verifies() {
    assert_eq!(validate(SIGNED_BOTH).unwrap().name_id, "alice@example.com");
}

#[test]
fn sha1_is_refused_unless_the_operator_allows_it() {
    let xml = include_str!("fixtures/sha1_assertion.xml");
    refused(xml, "not allowed");
    let allowed = validate_at(xml, IDP_PEM, now(), VerifyOptions { allow_sha1: true });
    assert_eq!(allowed.unwrap().name_id, "alice@example.com");
}

// --- tampering and forgery ------------------------------------------------------

#[test]
fn changed_name_id_breaks_the_signature() {
    refused(
        &SIGNED_ASSERTION.replace("alice@example.com", "mallory@example.com"),
        "digest",
    );
    refused(
        &SIGNED_RESPONSE.replace("alice@example.com", "mallory@example.com"),
        "digest",
    );
}

#[test]
fn changed_group_attribute_breaks_the_signature() {
    refused(
        &SIGNED_ASSERTION.replace("operators", "scada-admins2"),
        "digest",
    );
}

#[test]
fn changed_signature_value_is_refused() {
    let start = SIGNED_ASSERTION.find("<ds:SignatureValue>").unwrap() + "<ds:SignatureValue>".len();
    let mut bytes = SIGNED_ASSERTION.as_bytes().to_vec();
    bytes[start + 10] = if bytes[start + 10] == b'A' {
        b'B'
    } else {
        b'A'
    };
    refused(std::str::from_utf8(&bytes).unwrap(), "signature value");
}

#[test]
fn unsigned_response_is_refused() {
    refused(UNSIGNED, "neither the response nor the assertion is signed");
}

#[test]
fn stripping_the_signature_is_refused() {
    let a = SIGNED_ASSERTION.find("<ds:Signature").unwrap();
    let b = SIGNED_ASSERTION.find("</ds:Signature>").unwrap() + "</ds:Signature>".len();
    refused(
        &format!("{}{}", &SIGNED_ASSERTION[..a], &SIGNED_ASSERTION[b..]),
        "neither",
    );
}

#[test]
fn signature_from_an_unpinned_key_is_refused_even_with_its_certificate_in_keyinfo() {
    // The document's own KeyInfo names the IdP key; we trust only what is pinned.
    let r = validate_at(SIGNED_ASSERTION, OTHER_PEM, now(), VerifyOptions::default());
    assert!(r.unwrap_err().0.contains("signature value"));
}

#[test]
fn signature_wrapping_extra_assertion_is_refused() {
    // Forged assertion placed first, the genuinely signed one kept after it.
    let evil = r#"<saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_evil" Version="2.0"><saml:Issuer>https://idp.example.com/metadata</saml:Issuer><saml:Subject><saml:NameID>admin@example.com</saml:NameID></saml:Subject></saml:Assertion>"#;
    let at = SIGNED_ASSERTION.find("<saml:Assertion").unwrap();
    let wrapped = format!(
        "{}{}{}",
        &SIGNED_ASSERTION[..at],
        evil,
        &SIGNED_ASSERTION[at..]
    );
    refused(&wrapped, "unexpected assertion layout");
}

#[test]
fn signed_assertion_hidden_inside_an_advice_of_a_forged_one_is_refused() {
    let at = SIGNED_ASSERTION.find("<saml:Assertion").unwrap();
    let end = SIGNED_ASSERTION.rfind("</saml:Assertion>").unwrap() + "</saml:Assertion>".len();
    let genuine = &SIGNED_ASSERTION[at..end];
    let forged = format!(
        r#"<saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_evil" Version="2.0"><saml:Issuer>{IDP}</saml:Issuer><saml:Subject><saml:NameID>admin@example.com</saml:NameID></saml:Subject><saml:Advice>{genuine}</saml:Advice></saml:Assertion>"#
    );
    let wrapped = format!(
        "{}{}{}",
        &SIGNED_ASSERTION[..at],
        forged,
        &SIGNED_ASSERTION[end..]
    );
    refused(&wrapped, "assertion layout");
}

#[test]
fn signature_moved_to_the_response_no_longer_signs_the_assertion() {
    // Lift the assertion's signature up to the Response: its Reference still
    // names the assertion, but the parent is now the Response.
    let a = SIGNED_ASSERTION.find("<ds:Signature").unwrap();
    let b = SIGNED_ASSERTION.find("</ds:Signature>").unwrap() + "</ds:Signature>".len();
    let sig = &SIGNED_ASSERTION[a..b];
    let without = format!("{}{}", &SIGNED_ASSERTION[..a], &SIGNED_ASSERTION[b..]);
    let at = without.find("<samlp:Status>").unwrap();
    let moved = format!("{}{}{}", &without[..at], sig, &without[at..]);
    refused(&moved, "Reference URI does not name the signed element");
}

#[test]
fn reference_pointing_elsewhere_is_refused() {
    refused(
        &SIGNED_ASSERTION.replace("URI=\"#_assertion1\"", "URI=\"#_response1\""),
        "Reference URI",
    );
    refused(
        &SIGNED_ASSERTION.replace("URI=\"#_assertion1\"", "URI=\"\""),
        "Reference URI",
    );
}

#[test]
fn a_second_signature_anywhere_is_refused() {
    let a = SIGNED_ASSERTION.find("<ds:Signature").unwrap();
    let b = SIGNED_ASSERTION.find("</ds:Signature>").unwrap() + "</ds:Signature>".len();
    let sig = &SIGNED_ASSERTION[a..b];
    let at = SIGNED_ASSERTION.find("<saml:Conditions").unwrap();
    let doubled = format!(
        "{}{}{}",
        &SIGNED_ASSERTION[..at],
        sig,
        &SIGNED_ASSERTION[at..]
    );
    refused(&doubled, "multiple signatures");
}

#[test]
fn duplicate_ids_are_refused() {
    refused(
        &SIGNED_ASSERTION.replace("ID=\"_response1\"", "ID=\"_assertion1\""),
        "duplicate ID",
    );
}

#[test]
fn doctype_is_refused() {
    let xml = format!("<!DOCTYPE r [<!ENTITY x \"y\">]>{SIGNED_ASSERTION}");
    refused(&xml, "malformed XML");
}

#[test]
fn encrypted_assertions_are_refused_with_a_clear_message() {
    let xml = SIGNED_RESPONSE.replace(
        "</samlp:Response>",
        r#"<saml:EncryptedAssertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"/></samlp:Response>"#,
    );
    refused(&xml, "encrypted assertions");
}

#[test]
fn comment_inside_name_id_cannot_truncate_the_identity() {
    // Comments are dropped by canonicalization, so this still verifies; the
    // identity must be the whole text, not the first text node.
    let xml = SIGNED_ASSERTION.replace("alice@example.com", "alice@exa<!---->mple.com");
    assert_eq!(validate(&xml).unwrap().name_id, "alice@example.com");
}

// --- the checks inside a correctly signed assertion ----------------------------

#[test]
fn wrong_audience_is_refused() {
    refused(include_str!("fixtures/audience_wrong.xml"), "audience");
}

#[test]
fn wrong_recipient_is_refused() {
    refused(
        include_str!("fixtures/recipient_wrong.xml"),
        "bearer subject confirmation",
    );
}

#[test]
fn wrong_issuer_is_refused() {
    refused(include_str!("fixtures/issuer_wrong.xml"), "issuer");
}

#[test]
fn wrong_destination_is_refused() {
    refused(
        include_str!("fixtures/destination_wrong.xml"),
        "Destination",
    );
}

#[test]
fn expired_assertion_is_refused() {
    let later = parse_time("2030-01-01T00:10:00Z").unwrap();
    let r = validate_at(SIGNED_ASSERTION, IDP_PEM, later, VerifyOptions::default());
    assert!(r.unwrap_err().0.contains("expired"));
}

#[test]
fn assertion_from_the_future_is_refused() {
    let earlier = parse_time("2029-12-31T23:00:00Z").unwrap();
    let r = validate_at(SIGNED_ASSERTION, IDP_PEM, earlier, VerifyOptions::default());
    assert!(r.unwrap_err().0.contains("not valid yet"));
}

#[test]
fn clock_skew_is_tolerated_only_within_the_configured_window() {
    let noa = parse_time("2030-01-01T00:05:00Z").unwrap();
    let ok = validate_at(
        SIGNED_ASSERTION,
        IDP_PEM,
        noa + 30,
        VerifyOptions::default(),
    );
    assert!(ok.is_ok());
    let late = validate_at(
        SIGNED_ASSERTION,
        IDP_PEM,
        noa + 61,
        VerifyOptions::default(),
    );
    assert!(late.is_err());
}

// --- provider: request binding and replay ---------------------------------------

fn provider(allow_idp_initiated: bool) -> Arc<SamlProvider> {
    let mut cfg = SamlConfig::default();
    cfg.sp_entity_id = SP.into();
    cfg.acs_url = ACS.into();
    cfg.allow_idp_initiated = allow_idp_initiated;
    cfg.groups_attribute = "groups".into();
    SamlProvider::with_idp(
        cfg,
        IdpInfo {
            entity_id: IDP.into(),
            sso_url: "https://idp.example.com/sso".into(),
            keys: keys(IDP_PEM),
        },
    )
}

fn b64(xml: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(xml)
}

fn expect_pending(p: &SamlProvider, id: &str, binding: &str) {
    p.pending.insert(
        id.to_string(),
        PendingRequest {
            created: now(),
            return_to: "/screens/2".into(),
            browser_binding: binding.into(),
        },
    );
}

#[tokio::test]
async fn response_to_a_pending_request_logs_in_once() {
    let p = provider(false);
    expect_pending(&p, "_req1", "b1");
    let (a, pending) = p
        .process_response_at(&b64(SIGNED_ASSERTION), Some("b1"), true, now())
        .await
        .unwrap();
    assert_eq!(a.name_id, "alice@example.com");
    assert_eq!(pending.unwrap().return_to, "/screens/2");
    let id = p.identity(&a);
    assert_eq!(id.username, "alice@example.com");
    assert_eq!(id.groups, vec!["scada-admins", "operators"]);

    // The same response again, even with a fresh matching request, is a replay.
    expect_pending(&p, "_req1", "b1");
    let again = p
        .process_response_at(&b64(SIGNED_ASSERTION), Some("b1"), true, now())
        .await;
    assert!(again.unwrap_err().0.contains("already used"));
}

#[tokio::test]
async fn replayed_response_without_its_request_is_refused() {
    let p = provider(false);
    expect_pending(&p, "_req1", "b1");
    p.process_response_at(&b64(SIGNED_ASSERTION), Some("b1"), true, now())
        .await
        .unwrap();
    let again = p
        .process_response_at(&b64(SIGNED_ASSERTION), Some("b1"), true, now())
        .await;
    assert!(again.unwrap_err().0.contains("no pending request"));
}

#[tokio::test]
async fn unsolicited_response_is_refused_unless_idp_initiated_is_enabled() {
    let xml = include_str!("fixtures/idp_initiated.xml");
    let strict = provider(false);
    let r = strict
        .process_response_at(&b64(xml), None, true, now())
        .await;
    assert!(r.unwrap_err().0.contains("unsolicited"));

    let open = provider(true);
    let (a, pending) = open
        .process_response_at(&b64(xml), None, true, now())
        .await
        .unwrap();
    assert_eq!(a.name_id, "alice@example.com");
    assert!(pending.is_none());
    // IdP-initiated assertions are still one-time.
    let again = open.process_response_at(&b64(xml), None, true, now()).await;
    assert!(again.unwrap_err().0.contains("already used"));
}

#[tokio::test]
async fn response_to_someone_elses_request_is_refused() {
    let p = provider(false);
    expect_pending(&p, "_req1", "b1");
    // Signed for a request id (_other-request) this server never issued.
    let r = p
        .process_response_at(
            &b64(include_str!("fixtures/irt_other.xml")),
            Some("b1"),
            true,
            now(),
        )
        .await;
    assert!(r.unwrap_err().0.contains("no pending request"));
    // And the unrelated pending request is untouched.
    assert!(p.pending.contains_key("_req1"));
}

#[tokio::test]
async fn response_must_come_from_the_browser_that_started_the_login() {
    let p = provider(false);
    expect_pending(&p, "_req1", "b1");
    let r = p
        .process_response_at(&b64(SIGNED_ASSERTION), Some("someone-else"), true, now())
        .await;
    assert!(r.unwrap_err().0.contains("not requested by this browser"));
    // A failed attempt must not burn the legitimate user's pending login.
    assert!(
        p.process_response_at(&b64(SIGNED_ASSERTION), Some("b1"), true, now())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn garbage_responses_are_refused() {
    let p = provider(true);
    assert!(
        p.process_response_at("not base64 !!", None, false, now())
            .await
            .is_err()
    );
    assert!(
        p.process_response_at(&b64("<not-saml/>"), None, false, now())
            .await
            .is_err()
    );
    assert!(
        p.process_response_at(&b64("\u{0}\u{1}"), None, false, now())
            .await
            .is_err()
    );
}

// --- login redirect and metadata ----------------------------------------------

#[test]
fn begin_login_builds_a_redirect_binding_url_and_remembers_the_request() {
    let p = provider(false);
    let (url, binding) = p.begin_login("/screens/3").unwrap();
    assert!(url.starts_with("https://idp.example.com/sso?SAMLRequest="));
    assert_eq!(p.pending.len(), 1);

    let parsed = url::Url::parse(&url).unwrap();
    let q: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();
    let relay = &q["RelayState"];
    let pending = p.pending.get(relay).unwrap();
    assert_eq!(pending.return_to, "/screens/3");
    assert_eq!(pending.browser_binding, binding);

    // The request is raw-deflated base64 XML naming our ACS and entity ID.
    use std::io::Read;
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(&q["SAMLRequest"])
        .unwrap();
    let mut xml = String::new();
    flate2::read::DeflateDecoder::new(&compressed[..])
        .read_to_string(&mut xml)
        .unwrap();
    assert!(xml.contains(&format!("ID=\"{relay}\"")));
    assert!(xml.contains(&format!("AssertionConsumerServiceURL=\"{ACS}\"")));
    assert!(xml.contains(&format!("<saml:Issuer>{SP}</saml:Issuer>")));
    assert!(!q.contains_key("Signature"));
}

#[test]
fn pending_logins_are_bounded() {
    let p = provider(false);
    for i in 0..MAX_PENDING {
        p.pending.insert(
            format!("r{i}"),
            PendingRequest {
                created: now_secs(),
                return_to: "/".into(),
                browser_binding: String::new(),
            },
        );
    }
    assert!(p.begin_login("/").is_err());
}

#[test]
fn sp_metadata_names_the_acs_and_requires_signed_assertions() {
    let p = provider(false);
    let xml = p.metadata_xml();
    let doc = Document::parse(&xml).unwrap();
    let root = doc.root_element();
    assert_eq!(root.attribute("entityID"), Some(SP));
    let acs = doc
        .descendants()
        .find(|n| n.tag_name().name() == "AssertionConsumerService")
        .unwrap();
    assert_eq!(acs.attribute("Location"), Some(ACS));
    assert_eq!(acs.attribute("Binding"), Some(BINDING_POST));
    let sp = doc
        .descendants()
        .find(|n| n.tag_name().name() == "SPSSODescriptor")
        .unwrap();
    assert_eq!(sp.attribute("WantAssertionsSigned"), Some("true"));
}

// --- IdP metadata ----------------------------------------------------------------

fn metadata(cert_b64: &str) -> String {
    format!(
        r#"<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" entityID="{IDP}"><md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol"><md:KeyDescriptor use="signing"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert_b64}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor><md:KeyDescriptor use="encryption"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{other}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor><md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="https://idp.example.com/post"/><md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="https://idp.example.com/sso"/></md:IDPSSODescriptor></md:EntityDescriptor>"#,
        other = OTHER_PEM
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<String>(),
    )
}

#[test]
fn idp_metadata_yields_entity_redirect_url_and_only_signing_keys() {
    let cert: String = IDP_PEM
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect();
    let info = parse_idp_metadata(&metadata(&cert), "").unwrap();
    assert_eq!(info.entity_id, IDP);
    assert_eq!(info.sso_url, "https://idp.example.com/sso");
    assert_eq!(
        info.keys.len(),
        1,
        "the encryption certificate must not become a trusted signer"
    );
    assert_eq!(
        info.keys[0],
        xmldsig::rsa_key_from_certificate(IDP_PEM).unwrap()
    );
}

#[test]
fn idp_metadata_without_a_usable_certificate_is_refused() {
    assert!(parse_idp_metadata(&metadata("not a certificate"), "").is_err());
}

#[test]
fn federation_metadata_is_narrowed_by_entity_id() {
    let cert: String = IDP_PEM
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect();
    let one = metadata(&cert);
    let other = one.replace(IDP, "https://second-idp.example.com");
    let federation = format!(
        r#"<md:EntitiesDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata">{one}{other}</md:EntitiesDescriptor>"#
    );
    assert_eq!(
        parse_idp_metadata(&federation, "https://second-idp.example.com")
            .unwrap()
            .entity_id,
        "https://second-idp.example.com"
    );
    assert!(parse_idp_metadata(&federation, "https://nope.example.com").is_err());
}

#[test]
fn small_rsa_keys_are_not_trusted() {
    // A 1024-bit certificate must not become a trust anchor.
    let small = rsa::RsaPrivateKey::new(&mut rand_core::OsRng, 1024).unwrap();
    let err = xmldsig::rsa_public_key_checked(&rsa::RsaPublicKey::from(&small)).unwrap_err();
    assert!(err.0.contains("2048"));
}

#[tokio::test]
async fn idp_metadata_behind_a_private_ca_needs_ca_cert() {
    let cert: String = IDP_PEM
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect();
    let stub = crate::sso::test_tls::serve(vec![(
        "/metadata".into(),
        "application/samlmetadata+xml",
        metadata(&cert),
    )])
    .await;
    let url = format!("https://localhost:{}/metadata", stub.port);
    let cfg = |f: &dyn Fn(&mut SamlConfig)| {
        let mut c = SamlConfig::default();
        c.enabled = true;
        c.idp_metadata_url = url.clone();
        f(&mut c);
        c
    };
    let public_url = "https://scada.example.com";

    let err = SamlProvider::new(cfg(&|_| {}), public_url)
        .await
        .err()
        .unwrap();
    assert!(err.0.contains("fetching IdP metadata"), "{}", err.0);

    let pem = stub.ca_pem.clone();
    assert!(
        SamlProvider::new(cfg(&|c| c.ca_cert = pem.clone()), public_url)
            .await
            .is_ok()
    );

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ca.pem");
    std::fs::write(&path, &stub.ca_pem).unwrap();
    let file = path.to_string_lossy().into_owned();
    assert!(
        SamlProvider::new(cfg(&|c| c.ca_cert_file = file.clone()), public_url)
            .await
            .is_ok()
    );

    let err = SamlProvider::new(cfg(&|c| c.ca_cert = "not a certificate".into()), public_url)
        .await
        .err()
        .unwrap();
    assert!(err.0.contains("certificate"), "{}", err.0);
}
