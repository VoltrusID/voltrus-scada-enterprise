//! An HTTPS identity provider stub behind a private CA, for the tests that prove
//! `ca_cert` / `ca_cert_file` make SAML metadata, OIDC discovery and JWKS fetches trust it.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub struct TlsStub {
    pub port: u16,
    pub ca_pem: String,
}

/// Serve `routes` (path, content type, body) over TLS on a random localhost port; `{PORT}`
/// in a body becomes that port.
/// The certificate is for `localhost`, signed by a throwaway CA returned in `ca_pem`.
pub async fn serve(routes: Vec<(String, &'static str, String)>) -> TlsStub {
    use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, KeyPair};
    let ca_key = KeyPair::generate().unwrap();
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(DnType::CommonName, "Private IdP CA");
    let ca = ca_params.self_signed(&ca_key).unwrap();
    let key = KeyPair::generate().unwrap();
    let params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    let issuer = rcgen::Issuer::from_params(&ca_params, &ca_key);
    let cert = params.signed_by(&key, &issuer).unwrap();

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
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let routes = Arc::new(
        routes
            .into_iter()
            .map(|(p, c, b)| (p, c, b.replace("{PORT}", &port.to_string())))
            .collect::<Vec<_>>(),
    );
    tokio::spawn(async move {
        loop {
            let Ok((tcp, _)) = listener.accept().await else {
                return;
            };
            let acceptor = acceptor.clone();
            let routes = routes.clone();
            tokio::spawn(async move {
                let Ok(mut tls) = acceptor.accept(tcp).await else {
                    return;
                };
                let mut buf = vec![0u8; 8192];
                let n = tls.read(&mut buf).await.unwrap_or(0);
                let head = String::from_utf8_lossy(&buf[..n]);
                let path = head
                    .lines()
                    .next()
                    .and_then(|l| l.split(' ').nth(1))
                    .unwrap_or("/");
                let reply = match routes.iter().find(|(p, _, _)| p == path) {
                    Some((_, ctype, body)) => format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: {ctype}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                        body.len()
                    ),
                    None => {
                        "HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                            .to_string()
                    }
                };
                let _ = tls.write_all(reply.as_bytes()).await;
                let _ = tls.shutdown().await;
            });
        }
    });
    TlsStub {
        port,
        ca_pem: ca.pem(),
    }
}
