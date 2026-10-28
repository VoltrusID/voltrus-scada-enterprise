//! The authenticated, encrypted link between two nodes.
//!
//! * **Encryption and integrity**: TLS 1.3 (rustls, ring provider, no C
//!   dependency). The listener presents a certificate from `ha.tls` or, by
//!   default, a self-signed one generated at start. Nobody is asked to trust
//!   that certificate: identity does not come from it.
//! * **Authentication**: after the TLS handshake both ends prove they know the
//!   pre-shared secret with an HMAC-SHA256 that covers the TLS exporter secret
//!   of this very session (RFC 5705) and both nonces. Binding the proof to the
//!   session defeats a man in the middle that terminates TLS on both sides: the
//!   two sessions have different exporter secrets, so a relayed proof fails. It
//!   also makes a recorded handshake worthless for replay.
//!
//! The strength of the link is the strength of the secret: it must be long and
//! random (at least 32 characters; `openssl rand -hex 32` is right). Anyone
//! who can run a man in the middle and captures a proof can try secrets offline.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use hmac::{Hmac, Mac};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, WebPkiSupportedAlgorithms};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, ServerConfig, SignatureScheme};
use sha2::Sha256;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};

use super::wire::{Msg, read_msg, write_msg};

type HmacSha256 = Hmac<Sha256>;

const EXPORTER_LABEL: &[u8] = b"EXPORTER-voltrus-ha-link";
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

/// Links that failed authentication since start, for the status API.
static AUTH_FAILURES: AtomicU64 = AtomicU64::new(0);

pub fn auth_failures() -> u64 {
    AUTH_FAILURES.load(Ordering::Relaxed)
}

/// Shortest accepted secret.
pub const MIN_SECRET_LEN: usize = 32;

/// Everything needed to open links in either direction.
#[derive(Clone)]
pub struct LinkSecurity {
    node_id: String,
    secret: Arc<Vec<u8>>,
    acceptor: TlsAcceptor,
    connector: TlsConnector,
}

/// Accepts any certificate. The signatures of the TLS handshake are still
/// verified, so the session is really keyed with the presented certificate; but
/// the certificate says nothing about *who* the peer is, the shared secret does.
#[derive(Debug)]
struct AnyCertificate(WebPkiSupportedAlgorithms);

impl ServerCertVerifier for AnyCertificate {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.0)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.0)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.supported_schemes()
    }
}

fn load_identity(
    cert_path: &str,
    key_path: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let certs = CertificateDer::pem_file_iter(cert_path)
        .with_context(|| format!("Cannot read HA TLS certificate {cert_path}"))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("Bad HA TLS certificate in {cert_path}"))?;
    anyhow::ensure!(!certs.is_empty(), "No certificate in {cert_path}");
    let key = PrivateKeyDer::from_pem_file(key_path)
        .with_context(|| format!("Cannot read HA TLS key {key_path}"))?;
    Ok((certs, key))
}

fn generate_identity() -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let ck = rcgen::generate_simple_self_signed(vec!["voltrus-ha".to_string()])
        .context("Failed to generate the link certificate")?;
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(ck.signing_key.serialize_der()));
    Ok((vec![ck.cert.der().clone()], key))
}

impl LinkSecurity {
    /// `tls_files` is the optional `(cert_path, key_path)` of a certificate to
    /// present. Without it a certificate is generated for this process.
    pub fn new(node_id: &str, secret: &str, tls_files: Option<(&str, &str)>) -> Result<Self> {
        anyhow::ensure!(
            secret.len() >= MIN_SECRET_LEN,
            "ha.shared_secret must be at least {MIN_SECRET_LEN} characters (try: openssl rand -hex 32)"
        );
        let provider: Arc<CryptoProvider> = Arc::new(rustls::crypto::ring::default_provider());
        let versions = &[&rustls::version::TLS13];

        let (certs, key) = match tls_files {
            Some((c, k)) => load_identity(c, k)?,
            None => generate_identity()?,
        };
        let server = ServerConfig::builder_with_provider(provider.clone())
            .with_protocol_versions(versions)?
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .context("Invalid HA TLS certificate or key")?;
        let client = ClientConfig::builder_with_provider(provider.clone())
            .with_protocol_versions(versions)?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AnyCertificate(
                provider.signature_verification_algorithms,
            )))
            .with_no_client_auth();

        Ok(Self {
            node_id: node_id.to_string(),
            secret: Arc::new(secret.as_bytes().to_vec()),
            acceptor: TlsAcceptor::from(Arc::new(server)),
            connector: TlsConnector::from(Arc::new(client)),
        })
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Open a link to `addr` and authenticate it.
    pub async fn dial(&self, addr: &str, connect_timeout: Duration) -> Result<Link> {
        let tcp = timeout(connect_timeout, TcpStream::connect(addr))
            .await
            .context("connect timed out")?
            .with_context(|| format!("cannot connect to {addr}"))?;
        tcp.set_nodelay(true).ok();
        let peer_addr = tcp.peer_addr()?;
        let name = ServerName::try_from("voltrus-ha").expect("static name");
        let tls = timeout(HANDSHAKE_TIMEOUT, self.connector.connect(name, tcp))
            .await
            .context("TLS handshake timed out")?
            .context("TLS handshake failed")?;
        let ekm = exporter(tls.get_ref().1.export_keying_material(
            [0u8; 32],
            EXPORTER_LABEL,
            None,
        ))?;
        let mut stream: TlsStream<TcpStream> = tls.into();

        let mut nonce_c = [0u8; 16];
        rand::fill(&mut nonce_c);
        let mac_c = self.mac(b"init", &ekm, &[self.node_id.as_bytes(), &nonce_c]);
        write_msg(
            &mut stream,
            &Msg::AuthInit {
                node_id: self.node_id.clone(),
                nonce: nonce_c.to_vec(),
                mac: mac_c,
            },
        )
        .await?;
        let reply = timeout(HANDSHAKE_TIMEOUT, read_msg(&mut stream))
            .await
            .context("authentication timed out")??;
        let Msg::AuthReply {
            node_id,
            nonce,
            mac,
        } = reply
        else {
            bail!("peer did not answer the authentication challenge");
        };
        if self
            .verify(
                b"reply",
                &ekm,
                &[
                    self.node_id.as_bytes(),
                    &nonce_c,
                    node_id.as_bytes(),
                    &nonce,
                ],
                &mac,
            )
            .is_err()
        {
            AUTH_FAILURES.fetch_add(1, Ordering::Relaxed);
            bail!("peer {addr} failed authentication (wrong shared secret?)");
        }
        Ok(Link {
            peer_node_id: node_id,
            peer_addr,
            stream,
        })
    }

    /// Authenticate an accepted connection.
    pub async fn accept(&self, tcp: TcpStream) -> Result<Link> {
        tcp.set_nodelay(true).ok();
        let peer_addr = tcp.peer_addr()?;
        let tls = timeout(HANDSHAKE_TIMEOUT, self.acceptor.accept(tcp))
            .await
            .context("TLS handshake timed out")?
            .context("TLS handshake failed")?;
        let ekm = exporter(tls.get_ref().1.export_keying_material(
            [0u8; 32],
            EXPORTER_LABEL,
            None,
        ))?;
        let mut stream: TlsStream<TcpStream> = tls.into();

        let init = timeout(HANDSHAKE_TIMEOUT, read_msg(&mut stream))
            .await
            .context("authentication timed out")??;
        let Msg::AuthInit {
            node_id,
            nonce,
            mac,
        } = init
        else {
            bail!("peer {peer_addr} did not start with authentication");
        };
        if node_id.is_empty() || node_id.len() > 128 || nonce.len() != 16 {
            bail!("peer {peer_addr} sent a malformed authentication");
        }
        if self
            .verify(b"init", &ekm, &[node_id.as_bytes(), &nonce], &mac)
            .is_err()
        {
            AUTH_FAILURES.fetch_add(1, Ordering::Relaxed);
            bail!("peer {peer_addr} failed authentication (wrong shared secret?)");
        }
        let mut nonce_s = [0u8; 16];
        rand::fill(&mut nonce_s);
        let mac_s = self.mac(
            b"reply",
            &ekm,
            &[
                node_id.as_bytes(),
                &nonce,
                self.node_id.as_bytes(),
                &nonce_s,
            ],
        );
        write_msg(
            &mut stream,
            &Msg::AuthReply {
                node_id: self.node_id.clone(),
                nonce: nonce_s.to_vec(),
                mac: mac_s,
            },
        )
        .await?;
        Ok(Link {
            peer_node_id: node_id,
            peer_addr,
            stream,
        })
    }

    fn keyed(&self, label: &[u8], ekm: &[u8; 32], parts: &[&[u8]]) -> HmacSha256 {
        let mut m = HmacSha256::new_from_slice(&self.secret).expect("HMAC takes any key length");
        m.update(b"voltrus-ha-link/1");
        m.update(label);
        m.update(ekm);
        for p in parts {
            m.update(&(p.len() as u32).to_be_bytes());
            m.update(p);
        }
        m
    }

    fn mac(&self, label: &[u8], ekm: &[u8; 32], parts: &[&[u8]]) -> Vec<u8> {
        self.keyed(label, ekm, parts)
            .finalize()
            .into_bytes()
            .to_vec()
    }

    fn verify(&self, label: &[u8], ekm: &[u8; 32], parts: &[&[u8]], mac: &[u8]) -> Result<(), ()> {
        self.keyed(label, ekm, parts)
            .verify_slice(mac)
            .map_err(|_| ())
    }
}

fn exporter(r: std::result::Result<[u8; 32], rustls::Error>) -> Result<[u8; 32]> {
    r.map_err(|e| anyhow::anyhow!("TLS exporter unavailable: {e}"))
}

/// An authenticated connection.
pub struct Link {
    pub peer_node_id: String,
    pub peer_addr: SocketAddr,
    stream: TlsStream<TcpStream>,
}

impl Link {
    /// Split into a sender and a channel of received messages. Reading runs on
    /// its own task, so waiting for the next message is cancellation safe.
    pub fn into_peer(self, write_timeout: Duration) -> Peer {
        let (r, w): (ReadHalf<_>, WriteHalf<_>) = tokio::io::split(self.stream);
        let (tx, rx) = mpsc::channel(64);
        let reader = tokio::spawn(async move {
            let mut r = r;
            loop {
                let msg = read_msg(&mut r).await;
                let failed = msg.is_err();
                if tx.send(msg).await.is_err() || failed {
                    break;
                }
            }
        });
        Peer {
            node_id: self.peer_node_id,
            addr: self.peer_addr,
            writer: w,
            rx,
            reader,
            write_timeout,
        }
    }
}

pub struct Peer {
    pub node_id: String,
    pub addr: SocketAddr,
    writer: WriteHalf<TlsStream<TcpStream>>,
    pub rx: mpsc::Receiver<std::io::Result<Msg>>,
    reader: JoinHandle<()>,
    write_timeout: Duration,
}

impl Peer {
    pub async fn send(&mut self, msg: &Msg) -> Result<()> {
        timeout(self.write_timeout, write_msg(&mut self.writer, msg))
            .await
            .context("write to peer timed out")?
            .context("write to peer failed")
    }

    /// The next message, or `None` when the link is closed.
    pub async fn recv(&mut self) -> Option<std::io::Result<Msg>> {
        self.rx.recv().await
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.reader.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    const SECRET: &str = "0123456789abcdef0123456789abcdef0123456789abcdef";

    async fn listener() -> (TcpListener, String) {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap().to_string();
        (l, addr)
    }

    #[test]
    fn short_secrets_are_refused() {
        assert!(LinkSecurity::new("a", "too short", None).is_err());
        assert!(LinkSecurity::new("a", SECRET, None).is_ok());
    }

    #[tokio::test]
    async fn two_nodes_with_the_same_secret_authenticate_each_other() {
        let a = LinkSecurity::new("node-a", SECRET, None).unwrap();
        let b = LinkSecurity::new("node-b", SECRET, None).unwrap();
        let (l, addr) = listener().await;
        let server = tokio::spawn(async move {
            let (tcp, _) = l.accept().await.unwrap();
            b.accept(tcp).await
        });
        let link = a.dial(&addr, Duration::from_secs(2)).await.unwrap();
        assert_eq!(link.peer_node_id, "node-b");
        assert_eq!(server.await.unwrap().unwrap().peer_node_id, "node-a");

        // And frames flow over it.
        let (l, addr) = listener().await;
        let b = LinkSecurity::new("node-b", SECRET, None).unwrap();
        let echo = tokio::spawn(async move {
            let (tcp, _) = l.accept().await.unwrap();
            let mut p = b
                .accept(tcp)
                .await
                .unwrap()
                .into_peer(Duration::from_secs(2));
            let m = p.recv().await.unwrap().unwrap();
            p.send(&m).await.unwrap();
        });
        let mut p = a
            .dial(&addr, Duration::from_secs(2))
            .await
            .unwrap()
            .into_peer(Duration::from_secs(2));
        p.send(&Msg::Ack { seq: 42 }).await.unwrap();
        assert!(matches!(
            p.recv().await.unwrap().unwrap(),
            Msg::Ack { seq: 42 }
        ));
        echo.await.unwrap();
    }

    #[tokio::test]
    async fn a_wrong_secret_is_rejected_on_both_sides() {
        let a = LinkSecurity::new("node-a", SECRET, None).unwrap();
        let b = LinkSecurity::new("node-b", &SECRET.replace('0', "9"), None).unwrap();
        let before = auth_failures();
        let (l, addr) = listener().await;
        let server = tokio::spawn(async move {
            let (tcp, _) = l.accept().await.unwrap();
            b.accept(tcp).await.map(|_| ())
        });
        assert!(a.dial(&addr, Duration::from_secs(2)).await.is_err());
        assert!(server.await.unwrap().is_err());
        assert!(auth_failures() > before);
    }

    /// A man in the middle that terminates TLS on both legs and relays the
    /// frames cannot pass: the proof is bound to the exporter secret of each
    /// session, and the two sessions differ.
    #[tokio::test]
    async fn a_tls_terminating_relay_fails_authentication() {
        let a = LinkSecurity::new("node-a", SECRET, None).unwrap();
        let b = LinkSecurity::new("node-b", SECRET, None).unwrap();
        let mitm = LinkSecurity::new("mitm", SECRET, None).unwrap(); // only for its TLS config

        let (server_l, server_addr) = listener().await;
        let (mitm_l, mitm_addr) = listener().await;

        let b2 = b.clone();
        let server = tokio::spawn(async move {
            let (tcp, _) = server_l.accept().await.unwrap();
            b2.accept(tcp).await.map(|_| ())
        });
        // The relay: terminate the client's TLS, open its own to the server, and
        // pass the client's AuthInit through.
        tokio::spawn(async move {
            let (tcp, _) = mitm_l.accept().await.unwrap();
            let tls = mitm.acceptor.accept(tcp).await.unwrap();
            let mut client_side: TlsStream<TcpStream> = tls.into();
            let init = read_msg(&mut client_side).await.unwrap();

            let tcp = TcpStream::connect(&server_addr).await.unwrap();
            let name = ServerName::try_from("voltrus-ha").unwrap();
            let tls = mitm.connector.connect(name, tcp).await.unwrap();
            let mut server_side: TlsStream<TcpStream> = tls.into();
            write_msg(&mut server_side, &init).await.unwrap();
            if let Ok(reply) = read_msg(&mut server_side).await {
                let _ = write_msg(&mut client_side, &reply).await;
            }
        });

        assert!(a.dial(&mitm_addr, Duration::from_secs(2)).await.is_err());
        assert!(
            server.await.unwrap().is_err(),
            "the server rejects the relayed proof"
        );
    }
}
