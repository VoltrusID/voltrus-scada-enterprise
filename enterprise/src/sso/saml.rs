//! SAML 2.0 service provider: SP-initiated login (HTTP-Redirect AuthnRequest)
//! and the assertion consumer service (HTTP-POST), with XML signature
//! verification against certificates pinned from the IdP metadata.
//!
//! Trust model: only the signing certificates from the configured IdP metadata
//! (a URL that is re-fetched, a file, inline XML or explicit values) can vouch
//! for an assertion. Keys inside the response are ignored. Claims are read only
//! from the element a verified signature is attached to.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use base64::Engine;
use dashmap::DashMap;
use rand::Rng;
use roxmltree::{Document, Node, ParsingOptions};
use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::config::{RoleMapping, secret};
use super::xmldsig::{
    self, NS_DS, SigError, VerifyOptions, children, ensure_unique_ids, is_element,
    verify_enveloped_signature,
};

const NS_SAMLP: &str = "urn:oasis:names:tc:SAML:2.0:protocol";
const NS_SAML: &str = "urn:oasis:names:tc:SAML:2.0:assertion";
const NS_MD: &str = "urn:oasis:names:tc:SAML:2.0:metadata";
const STATUS_SUCCESS: &str = "urn:oasis:names:tc:SAML:2.0:status:Success";
const CM_BEARER: &str = "urn:oasis:names:tc:SAML:2.0:cm:bearer";
const BINDING_REDIRECT: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect";
const BINDING_POST: &str = "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST";

/// Largest SAMLResponse accepted, decoded.
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
/// How long a started login stays valid.
const REQUEST_TTL_SECS: i64 = 600;
/// Pending logins held at once; bounds memory against unauthenticated floods.
const MAX_PENDING: usize = 10_000;

/// `sso.saml` in the configuration file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SamlConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Button text on the login page.
    #[serde(default = "default_label")]
    pub label: String,
    /// Our entity ID. Default `<public_url>/api/v1/auth/saml/metadata`.
    #[serde(default)]
    pub sp_entity_id: String,
    /// Assertion consumer service URL. Default `<public_url>/api/v1/auth/saml/acs`.
    #[serde(default)]
    pub acs_url: String,

    /// IdP metadata, fetched over HTTPS and refreshed.
    #[serde(default)]
    pub idp_metadata_url: String,
    /// IdP metadata read from a file, at startup and on every refresh.
    #[serde(default)]
    pub idp_metadata_file: String,
    /// IdP metadata XML inline.
    #[serde(default)]
    pub idp_metadata_xml: String,
    /// Without metadata: the IdP entity ID, redirect-binding SSO URL and
    /// signing certificate(s), as PEM or bare base64.
    #[serde(default)]
    pub idp_entity_id: String,
    #[serde(default)]
    pub idp_sso_url: String,
    #[serde(default)]
    pub idp_certificate: String,
    #[serde(default)]
    pub idp_certificates: Vec<String>,
    /// How often IdP metadata is re-fetched, so certificate rollover needs no restart.
    /// With `idp_entity_id` set, it also picks one entity from a federation document.
    #[serde(default = "default_refresh_hours")]
    pub metadata_refresh_hours: u64,

    /// Accept RSA-SHA1 signatures and SHA-1 digests (old IdPs only).
    #[serde(default)]
    pub allow_sha1: bool,
    /// Accept responses that answer no request of ours (IdP-initiated login).
    #[serde(default)]
    pub allow_idp_initiated: bool,
    #[serde(default = "default_skew")]
    pub clock_skew_secs: i64,
    #[serde(default = "default_name_id_format")]
    pub name_id_format: String,

    /// Attribute carrying the login name; empty uses the NameID.
    #[serde(default)]
    pub username_attribute: String,
    /// Attribute carrying group or role values (several values allowed).
    #[serde(default)]
    pub groups_attribute: String,
    #[serde(default)]
    pub email_attribute: String,
    #[serde(default)]
    pub display_name_attribute: String,
    #[serde(default)]
    pub role_mapping: RoleMapping,

    /// Sign AuthnRequests (HTTP-Redirect signature, RSA-SHA256). Needs a key.
    #[serde(default)]
    pub sign_authn_requests: bool,
    /// SP certificate (PEM or base64), published in the SP metadata.
    #[serde(default)]
    pub sp_certificate: String,
    #[serde(default)]
    pub sp_certificate_file: String,
    /// SP private key, PKCS#8 or PKCS#1 PEM.
    #[serde(default)]
    pub sp_private_key: String,
    #[serde(default)]
    pub sp_private_key_file: String,
    #[serde(default)]
    pub sp_private_key_env: String,
}

fn default_label() -> String {
    "SAML 2.0".to_string()
}
fn default_refresh_hours() -> u64 {
    12
}
fn default_skew() -> i64 {
    60
}
fn default_name_id_format() -> String {
    "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified".to_string()
}

impl Default for SamlConfig {
    fn default() -> Self {
        serde_yaml_neo::from_str("{}").expect("empty SAML config deserializes")
    }
}

/// What we know about the IdP: identity, where to send users and whom to trust.
#[derive(Clone)]
pub struct IdpInfo {
    pub entity_id: String,
    pub sso_url: String,
    pub keys: Vec<RsaPublicKey>,
}

/// A login this server started and has not yet seen answered.
#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub created: i64,
    pub return_to: String,
    /// Random value also set as a cookie, tying the response to the browser that asked.
    pub browser_binding: String,
}

/// Why a SAML response was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SamlError(pub String);

impl std::fmt::Display for SamlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SamlError {}

impl From<SigError> for SamlError {
    fn from(e: SigError) -> Self {
        SamlError(format!("signature: {}", e.0))
    }
}

fn fail<T>(msg: impl Into<String>) -> Result<T, SamlError> {
    Err(SamlError(msg.into()))
}

/// The claims of an assertion whose signature and conditions were verified.
#[derive(Debug, Clone)]
pub struct ValidatedAssertion {
    pub assertion_id: String,
    pub name_id: String,
    pub session_index: Option<String>,
    /// Attribute values by `Name` and by `FriendlyName`.
    pub attributes: BTreeMap<String, Vec<String>>,
    /// The request this answers, from the signed subject confirmation.
    pub in_response_to: Option<String>,
    /// Unix time after which the assertion is dead (with skew); bounds the replay cache.
    pub expires_at: i64,
}

/// Everything the response is checked against.
pub struct ValidationContext<'a> {
    pub sp_entity_id: &'a str,
    pub acs_url: &'a str,
    pub idp_entity_id: &'a str,
    pub idp_keys: &'a [RsaPublicKey],
    pub now: i64,
    pub skew: i64,
    pub opts: VerifyOptions,
}

// ---------------------------------------------------------------------------
// Response validation
// ---------------------------------------------------------------------------

fn parse_time(s: &str) -> Result<i64, SamlError> {
    OffsetDateTime::parse(s.trim(), &Rfc3339)
        .map(|t| t.unix_timestamp())
        .map_err(|_| SamlError(format!("bad timestamp '{s}'")))
}

fn text(node: Node) -> String {
    node.descendants()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect::<String>()
        .trim()
        .to_string()
}

fn child<'a, 'i>(
    node: Node<'a, 'i>,
    ns: &'static str,
    local: &'static str,
) -> Option<Node<'a, 'i>> {
    children(node, ns, local).next()
}

/// Validate a decoded SAML `Response`: signature, issuer, destination, status,
/// subject confirmation, conditions and audience. On success returns the claims
/// of the one verified assertion. Request and replay bookkeeping is the
/// caller's ([`SamlProvider::process_response`]).
pub fn validate_response(
    xml: &str,
    ctx: &ValidationContext,
) -> Result<ValidatedAssertion, SamlError> {
    if xml.len() > MAX_RESPONSE_BYTES {
        return fail("response too large");
    }
    // No DTDs: no entity expansion and no external entities.
    let options = ParsingOptions {
        allow_dtd: false,
        nodes_limit: 100_000,
    };
    let doc = Document::parse_with_options(xml, options)
        .map_err(|e| SamlError(format!("malformed XML: {e}")))?;
    let root = doc.root_element();
    if !is_element(root, NS_SAMLP, "Response") {
        return fail("not a SAML Response");
    }
    ensure_unique_ids(&doc)?;

    // Exactly one assertion in the whole document, directly under the Response.
    // Nested or extra assertions (Advice, Extensions) are never read, and their
    // presence is how most wrapping attacks smuggle content in.
    if doc
        .descendants()
        .any(|n| is_element(n, NS_SAML, "EncryptedAssertion"))
    {
        return fail(
            "encrypted assertions are not supported; disable assertion encryption at the IdP",
        );
    }
    let assertion_count = doc
        .descendants()
        .filter(|n| is_element(*n, NS_SAML, "Assertion"))
        .count();
    let assertion = match (assertion_count, child(root, NS_SAML, "Assertion")) {
        (1, Some(a)) => a,
        (0, _) => return fail("response carries no assertion"),
        _ => return fail("unexpected assertion layout"),
    };

    // Signatures: only as direct children of the Response or the Assertion, and
    // no others anywhere else in the document.
    let response_sigs: Vec<Node> = children(root, NS_DS, "Signature").collect();
    let assertion_sigs: Vec<Node> = children(assertion, NS_DS, "Signature").collect();
    let total_sigs = doc
        .descendants()
        .filter(|n| is_element(*n, NS_DS, "Signature"))
        .count();
    if response_sigs.len() > 1 || assertion_sigs.len() > 1 {
        return fail("multiple signatures on one element");
    }
    if total_sigs != response_sigs.len() + assertion_sigs.len() {
        return fail("signature in an unexpected position");
    }
    let mut response_signed = false;
    let mut assertion_signed = false;
    if let Some(sig) = response_sigs.first() {
        let verified = verify_enveloped_signature(xml, *sig, ctx.idp_keys, ctx.opts)?;
        response_signed = verified.id() == root.id();
    }
    if let Some(sig) = assertion_sigs.first() {
        let verified = verify_enveloped_signature(xml, *sig, ctx.idp_keys, ctx.opts)?;
        assertion_signed = verified.id() == assertion.id();
    }
    if !response_signed && !assertion_signed {
        return fail("neither the response nor the assertion is signed");
    }

    // Response-level checks. Trustworthy when the Response is signed; when only
    // the Assertion is signed they still must not contradict, and everything
    // that matters is re-checked inside the assertion.
    let status = child(root, NS_SAMLP, "Status")
        .and_then(|s| child(s, NS_SAMLP, "StatusCode"))
        .and_then(|c| c.attribute("Value"));
    if status != Some(STATUS_SUCCESS) {
        return fail(format!(
            "IdP reported failure: {}",
            status.unwrap_or("no status")
        ));
    }
    if let Some(issuer) = child(root, NS_SAML, "Issuer")
        && text(issuer) != ctx.idp_entity_id
    {
        return fail("response issuer is not the configured IdP");
    }
    match root.attribute("Destination") {
        Some(d) if d != ctx.acs_url => return fail("response Destination is not our ACS URL"),
        None if response_signed => return fail("signed response has no Destination"),
        _ => {}
    }

    // Assertion issuer.
    match child(assertion, NS_SAML, "Issuer") {
        Some(i) if text(i) == ctx.idp_entity_id => {}
        Some(_) => return fail("assertion issuer is not the configured IdP"),
        None => return fail("assertion has no Issuer"),
    }
    let assertion_id = assertion
        .attribute("ID")
        .ok_or_else(|| SamlError("assertion has no ID".into()))?
        .to_string();

    // Conditions: validity window and audience.
    let conditions = child(assertion, NS_SAML, "Conditions")
        .ok_or_else(|| SamlError("assertion has no Conditions".into()))?;
    let mut expires_at = i64::MAX;
    if let Some(nb) = conditions.attribute("NotBefore")
        && parse_time(nb)? > ctx.now + ctx.skew
    {
        return fail("assertion is not valid yet");
    }
    if let Some(noa) = conditions.attribute("NotOnOrAfter") {
        let noa = parse_time(noa)?;
        if ctx.now - ctx.skew >= noa {
            return fail("assertion has expired");
        }
        expires_at = expires_at.min(noa);
    }
    let mut audience_restrictions = 0;
    for cond in conditions.children().filter(|c| c.is_element()) {
        if is_element(cond, NS_SAML, "AudienceRestriction") {
            audience_restrictions += 1;
            let ok = children(cond, NS_SAML, "Audience").any(|a| text(a) == ctx.sp_entity_id);
            if !ok {
                return fail("assertion audience does not include this service");
            }
        } else if !is_element(cond, NS_SAML, "OneTimeUse")
            && !is_element(cond, NS_SAML, "ProxyRestriction")
        {
            // An extension condition we cannot evaluate must not be ignored.
            return fail(format!(
                "unsupported condition <{}>",
                cond.tag_name().name()
            ));
        }
    }
    if audience_restrictions == 0 {
        return fail("assertion has no audience restriction");
    }

    // Subject: NameID and a bearer confirmation addressed to us.
    let subject = child(assertion, NS_SAML, "Subject")
        .ok_or_else(|| SamlError("assertion has no Subject".into()))?;
    let name_id = child(subject, NS_SAML, "NameID")
        .map(text)
        .filter(|n| !n.is_empty())
        .ok_or_else(|| SamlError("assertion has no NameID".into()))?;
    let mut in_response_to: Option<String> = None;
    let mut confirmed = false;
    for sc in children(subject, NS_SAML, "SubjectConfirmation") {
        if sc.attribute("Method") != Some(CM_BEARER) {
            continue;
        }
        let Some(data) = child(sc, NS_SAML, "SubjectConfirmationData") else {
            continue;
        };
        if data.attribute("Recipient") != Some(ctx.acs_url) {
            continue;
        }
        let Some(noa) = data
            .attribute("NotOnOrAfter")
            .and_then(|v| parse_time(v).ok())
        else {
            continue;
        };
        if ctx.now - ctx.skew >= noa {
            continue;
        }
        if let Some(nb) = data.attribute("NotBefore").and_then(|v| parse_time(v).ok())
            && nb > ctx.now + ctx.skew
        {
            continue;
        }
        confirmed = true;
        in_response_to = data.attribute("InResponseTo").map(str::to_string);
        expires_at = expires_at.min(noa);
        break;
    }
    if !confirmed {
        return fail("no valid bearer subject confirmation for this service");
    }
    // A signed Response must agree about which request it answers.
    if response_signed
        && let Some(r) = root.attribute("InResponseTo")
        && in_response_to.as_deref().is_some_and(|a| a != r)
    {
        return fail("InResponseTo disagrees between response and assertion");
    }
    if in_response_to.is_none() && response_signed {
        in_response_to = root.attribute("InResponseTo").map(str::to_string);
    }

    let session_index = children(assertion, NS_SAML, "AuthnStatement")
        .find_map(|s| s.attribute("SessionIndex").map(str::to_string));

    let mut attributes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for stmt in children(assertion, NS_SAML, "AttributeStatement") {
        for attr in children(stmt, NS_SAML, "Attribute") {
            let values: Vec<String> = children(attr, NS_SAML, "AttributeValue")
                .map(text)
                .filter(|v| !v.is_empty())
                .collect();
            for key in [attr.attribute("Name"), attr.attribute("FriendlyName")]
                .into_iter()
                .flatten()
            {
                attributes
                    .entry(key.to_string())
                    .or_default()
                    .extend(values.clone());
            }
        }
    }

    Ok(ValidatedAssertion {
        assertion_id,
        name_id,
        session_index,
        attributes,
        in_response_to,
        expires_at,
    })
}

// ---------------------------------------------------------------------------
// IdP metadata
// ---------------------------------------------------------------------------

/// Extract the IdP identity, redirect-binding SSO URL and signing keys from
/// SAML metadata. `entity_hint` selects one entity in a federation document.
pub fn parse_idp_metadata(xml: &str, entity_hint: &str) -> Result<IdpInfo, SamlError> {
    let doc = Document::parse_with_options(
        xml,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: 500_000,
        },
    )
    .map_err(|e| SamlError(format!("malformed metadata XML: {e}")))?;
    let root = doc.root_element();
    let entities: Vec<Node> = if is_element(root, NS_MD, "EntityDescriptor") {
        vec![root]
    } else if is_element(root, NS_MD, "EntitiesDescriptor") {
        doc.descendants()
            .filter(|n| is_element(*n, NS_MD, "EntityDescriptor"))
            .collect()
    } else {
        return fail("not SAML metadata");
    };
    let entity = entities
        .iter()
        .copied()
        .filter(|e| child(*e, NS_MD, "IDPSSODescriptor").is_some())
        .find(|e| entity_hint.is_empty() || e.attribute("entityID") == Some(entity_hint))
        .ok_or_else(|| SamlError("metadata has no matching IdP entity".into()))?;
    let idp = child(entity, NS_MD, "IDPSSODescriptor").expect("filtered above");

    let sso_url = children(idp, NS_MD, "SingleSignOnService")
        .find(|s| s.attribute("Binding") == Some(BINDING_REDIRECT))
        .and_then(|s| s.attribute("Location"))
        .ok_or_else(|| SamlError("IdP metadata has no HTTP-Redirect SingleSignOnService".into()))?
        .to_string();

    let mut keys = Vec::new();
    for kd in children(idp, NS_MD, "KeyDescriptor") {
        if let Some(u) = kd.attribute("use")
            && u != "signing"
        {
            continue;
        }
        for cert in kd
            .descendants()
            .filter(|n| is_element(*n, NS_DS, "X509Certificate"))
        {
            match xmldsig::rsa_key_from_certificate(&text(cert)) {
                Ok(k) => keys.push(k),
                Err(e) => tracing::warn!("SAML metadata: skipping a signing certificate: {e}"),
            }
        }
    }
    if keys.is_empty() {
        return fail("IdP metadata has no usable RSA signing certificate");
    }
    Ok(IdpInfo {
        entity_id: entity.attribute("entityID").unwrap_or_default().to_string(),
        sso_url,
        keys,
    })
}

fn idp_from_explicit(cfg: &SamlConfig) -> Result<IdpInfo, SamlError> {
    let mut certs: Vec<&str> = cfg.idp_certificates.iter().map(String::as_str).collect();
    if !cfg.idp_certificate.trim().is_empty() {
        certs.push(&cfg.idp_certificate);
    }
    let mut keys = Vec::new();
    for c in certs {
        keys.push(xmldsig::rsa_key_from_certificate(c)?);
    }
    if cfg.idp_entity_id.is_empty() || cfg.idp_sso_url.is_empty() || keys.is_empty() {
        return fail("saml needs idp metadata, or idp_entity_id, idp_sso_url and idp_certificate");
    }
    Ok(IdpInfo {
        entity_id: cfg.idp_entity_id.clone(),
        sso_url: cfg.idp_sso_url.clone(),
        keys,
    })
}

// ---------------------------------------------------------------------------
// Requests (SP -> IdP)
// ---------------------------------------------------------------------------

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn iso_now(now: i64) -> String {
    OffsetDateTime::from_unix_timestamp(now)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .format(&Rfc3339)
        .unwrap_or_default()
}

/// An `AuthnRequest` document asking the IdP for a login answered at `acs_url`.
pub fn authn_request_xml(
    request_id: &str,
    now: i64,
    sp_entity_id: &str,
    acs_url: &str,
    idp_sso_url: &str,
    name_id_format: &str,
) -> String {
    let policy = if name_id_format.is_empty() {
        String::new()
    } else {
        format!(
            r#"<samlp:NameIDPolicy Format="{}" AllowCreate="true"/>"#,
            xml_escape(name_id_format)
        )
    };
    format!(
        r#"<samlp:AuthnRequest xmlns:samlp="{NS_SAMLP}" xmlns:saml="{NS_SAML}" ID="{id}" Version="2.0" IssueInstant="{instant}" Destination="{dest}" ProtocolBinding="{BINDING_POST}" AssertionConsumerServiceURL="{acs}"><saml:Issuer>{issuer}</saml:Issuer>{policy}</samlp:AuthnRequest>"#,
        id = xml_escape(request_id),
        instant = iso_now(now),
        dest = xml_escape(idp_sso_url),
        acs = xml_escape(acs_url),
        issuer = xml_escape(sp_entity_id),
    )
}

fn form_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// The HTTP-Redirect binding URL for an `AuthnRequest`: raw-deflated, base64,
/// URL-encoded, optionally signed over the query string (RSA-SHA256).
pub fn redirect_url(
    idp_sso_url: &str,
    authn_request: &str,
    relay_state: &str,
    signing_key: Option<&rsa::RsaPrivateKey>,
) -> Result<String, SamlError> {
    use flate2::Compression;
    use flate2::write::DeflateEncoder;
    use std::io::Write;

    let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
    enc.write_all(authn_request.as_bytes())
        .map_err(|e| SamlError(format!("deflate failed: {e}")))?;
    let deflated = enc
        .finish()
        .map_err(|e| SamlError(format!("deflate failed: {e}")))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(deflated);

    let mut query = format!(
        "SAMLRequest={}&RelayState={}",
        form_encode(&b64),
        form_encode(relay_state)
    );
    if let Some(key) = signing_key {
        use rsa::pkcs1v15::SigningKey;
        use rsa::sha2::Sha256;
        use rsa::signature::{SignatureEncoding, Signer};
        const SIG_ALG: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
        query.push_str(&format!("&SigAlg={}", form_encode(SIG_ALG)));
        let sig = SigningKey::<Sha256>::new(key.clone()).sign(query.as_bytes());
        query.push_str(&format!(
            "&Signature={}",
            form_encode(&base64::engine::general_purpose::STANDARD.encode(sig.to_bytes()))
        ));
    }
    let sep = if idp_sso_url.contains('?') { '&' } else { '?' };
    Ok(format!("{idp_sso_url}{sep}{query}"))
}

/// SP metadata to register with the IdP.
pub fn sp_metadata_xml(cfg: &SamlConfig, sp_certificate: &str) -> String {
    let cert: String = sp_certificate
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("-----"))
        .collect();
    let key_descriptor = if cert.is_empty() {
        String::new()
    } else {
        format!(
            r#"<md:KeyDescriptor use="signing"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>"#
        )
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><md:EntityDescriptor xmlns:md="{NS_MD}" xmlns:ds="{NS_DS}" entityID="{entity}"><md:SPSSODescriptor AuthnRequestsSigned="{signed}" WantAssertionsSigned="true" protocolSupportEnumeration="{NS_SAMLP}">{key_descriptor}<md:NameIDFormat>{nameid}</md:NameIDFormat><md:AssertionConsumerService Binding="{BINDING_POST}" Location="{acs}" index="0" isDefault="true"/></md:SPSSODescriptor></md:EntityDescriptor>"#,
        entity = xml_escape(&cfg.sp_entity_id),
        signed = cfg.sign_authn_requests,
        nameid = xml_escape(&cfg.name_id_format),
        acs = xml_escape(&cfg.acs_url),
    )
}

// ---------------------------------------------------------------------------
// Provider: configuration, IdP trust, request and replay bookkeeping
// ---------------------------------------------------------------------------

/// Where IdP metadata comes from, resolved once at startup.
enum MetadataSource {
    Url(String),
    File(String),
    Inline(String),
    Explicit,
}

pub struct SamlProvider {
    pub cfg: SamlConfig,
    source: MetadataSource,
    idp: RwLock<IdpInfo>,
    last_refresh: RwLock<i64>,
    sp_certificate: String,
    signing_key: Option<rsa::RsaPrivateKey>,
    pending: DashMap<String, PendingRequest>,
    /// Consumed assertion IDs with the time they stop being replayable.
    seen: DashMap<String, i64>,
    http: reqwest::Client,
}

fn read_file(path: &str, what: &str) -> Result<String, SamlError> {
    std::fs::read_to_string(path).map_err(|e| SamlError(format!("cannot read {what} {path}: {e}")))
}

fn now_secs() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

fn random_token(bytes: usize) -> String {
    let b: Vec<u8> = (0..bytes).map(|_| rand::rng().random()).collect();
    hex::encode(b)
}

impl SamlProvider {
    /// Resolve the configuration (URLs defaulted from `public_url`, secrets,
    /// IdP metadata) into a provider. Fails with a message the operator can act on.
    pub async fn new(mut cfg: SamlConfig, public_url: &str) -> Result<Arc<Self>, SamlError> {
        cfg.role_mapping
            .validate()
            .map_err(|e| SamlError(format!("saml.role_mapping: {e}")))?;
        if cfg.sp_entity_id.is_empty() || cfg.acs_url.is_empty() {
            if public_url.is_empty() {
                return fail("saml needs sso.public_url (or sp_entity_id and acs_url)");
            }
            if cfg.sp_entity_id.is_empty() {
                cfg.sp_entity_id = format!("{public_url}/api/v1/auth/saml/metadata");
            }
            if cfg.acs_url.is_empty() {
                cfg.acs_url = format!("{public_url}/api/v1/auth/saml/acs");
            }
        }

        let source = if !cfg.idp_metadata_url.is_empty() {
            MetadataSource::Url(cfg.idp_metadata_url.clone())
        } else if !cfg.idp_metadata_file.is_empty() {
            MetadataSource::File(cfg.idp_metadata_file.clone())
        } else if !cfg.idp_metadata_xml.trim().is_empty() {
            MetadataSource::Inline(cfg.idp_metadata_xml.clone())
        } else {
            MetadataSource::Explicit
        };

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()
            .map_err(|e| SamlError(format!("http client: {e}")))?;

        let sp_certificate = if !cfg.sp_certificate.is_empty() {
            cfg.sp_certificate.clone()
        } else if !cfg.sp_certificate_file.is_empty() {
            read_file(&cfg.sp_certificate_file, "sp_certificate_file")?
        } else {
            String::new()
        };
        let key_pem = {
            let inline = secret(&cfg.sp_private_key, &cfg.sp_private_key_env);
            if !inline.is_empty() {
                inline
            } else if !cfg.sp_private_key_file.is_empty() {
                read_file(&cfg.sp_private_key_file, "sp_private_key_file")?
            } else {
                String::new()
            }
        };
        let signing_key = if key_pem.is_empty() {
            None
        } else {
            use rsa::pkcs8::DecodePrivateKey;
            Some(
                rsa::RsaPrivateKey::from_pkcs8_pem(&key_pem)
                    .or_else(|_| {
                        use rsa::pkcs1::DecodeRsaPrivateKey;
                        rsa::RsaPrivateKey::from_pkcs1_pem(&key_pem)
                    })
                    .map_err(|e| {
                        SamlError(format!("sp_private_key is not a valid RSA key: {e}"))
                    })?,
            )
        };
        if cfg.sign_authn_requests && signing_key.is_none() {
            return fail("sign_authn_requests needs sp_private_key or sp_private_key_file");
        }

        let provider = SamlProvider {
            source,
            idp: RwLock::new(IdpInfo {
                entity_id: String::new(),
                sso_url: String::new(),
                keys: Vec::new(),
            }),
            last_refresh: RwLock::new(0),
            sp_certificate,
            signing_key,
            pending: DashMap::new(),
            seen: DashMap::new(),
            http,
            cfg,
        };
        provider.refresh_idp().await?;
        Ok(Arc::new(provider))
    }

    /// A provider with the IdP already known and no network or files involved.
    pub fn with_idp(cfg: SamlConfig, idp: IdpInfo) -> Arc<Self> {
        Arc::new(SamlProvider {
            source: MetadataSource::Explicit,
            idp: RwLock::new(idp),
            last_refresh: RwLock::new(now_secs()),
            sp_certificate: String::new(),
            signing_key: None,
            pending: DashMap::new(),
            seen: DashMap::new(),
            http: reqwest::Client::new(),
            cfg,
        })
    }

    /// Load the IdP's identity and keys from the configured source. The previous
    /// trust stays in place when this fails.
    pub async fn refresh_idp(&self) -> Result<(), SamlError> {
        let info = match &self.source {
            MetadataSource::Explicit => idp_from_explicit(&self.cfg)?,
            MetadataSource::Inline(xml) => parse_idp_metadata(xml, &self.cfg.idp_entity_id)?,
            MetadataSource::File(path) => parse_idp_metadata(
                &read_file(path, "idp_metadata_file")?,
                &self.cfg.idp_entity_id,
            )?,
            MetadataSource::Url(url) => {
                if !url.starts_with("https://") {
                    return fail("idp_metadata_url must be https");
                }
                let resp = self
                    .http
                    .get(url)
                    .send()
                    .await
                    .and_then(|r| r.error_for_status())
                    .map_err(|e| SamlError(format!("fetching IdP metadata failed: {e}")))?;
                let body = resp
                    .text()
                    .await
                    .map_err(|e| SamlError(format!("reading IdP metadata failed: {e}")))?;
                parse_idp_metadata(&body, &self.cfg.idp_entity_id)?
            }
        };
        *self.idp.write().unwrap_or_else(|e| e.into_inner()) = info;
        *self.last_refresh.write().unwrap_or_else(|e| e.into_inner()) = now_secs();
        Ok(())
    }

    /// Re-fetch the metadata on a schedule so certificate rollover needs no restart.
    pub fn spawn_refresh(self: &Arc<Self>) {
        if matches!(
            self.source,
            MetadataSource::Explicit | MetadataSource::Inline(_)
        ) {
            return;
        }
        let me = self.clone();
        tokio::spawn(async move {
            let period =
                std::time::Duration::from_secs(me.cfg.metadata_refresh_hours.max(1) * 3600);
            let mut tick = tokio::time::interval(period);
            tick.tick().await;
            loop {
                tick.tick().await;
                if let Err(e) = me.refresh_idp().await {
                    tracing::warn!(
                        "SAML IdP metadata refresh failed, keeping the previous keys: {e}"
                    );
                }
            }
        });
    }

    fn idp(&self) -> IdpInfo {
        self.idp.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// SP metadata document for the IdP administrator.
    pub fn metadata_xml(&self) -> String {
        sp_metadata_xml(&self.cfg, &self.sp_certificate)
    }

    /// Start a login: remember the request and return the IdP redirect URL plus
    /// the value to bind to the browser.
    pub fn begin_login(&self, return_to: &str) -> Result<(String, String), SamlError> {
        let now = now_secs();
        self.pending
            .retain(|_, p| now - p.created < REQUEST_TTL_SECS);
        if self.pending.len() >= MAX_PENDING {
            return fail("too many logins in progress");
        }
        let idp = self.idp();
        let request_id = format!("_{}", random_token(16));
        let binding = random_token(16);
        let xml = authn_request_xml(
            &request_id,
            now,
            &self.cfg.sp_entity_id,
            &self.cfg.acs_url,
            &idp.sso_url,
            &self.cfg.name_id_format,
        );
        let signing_key = if self.cfg.sign_authn_requests {
            self.signing_key.as_ref()
        } else {
            None
        };
        let url = redirect_url(&idp.sso_url, &xml, &request_id, signing_key)?;
        self.pending.insert(
            request_id,
            PendingRequest {
                created: now,
                return_to: return_to.to_string(),
                browser_binding: binding.clone(),
            },
        );
        Ok((url, binding))
    }

    /// Validate a base64 `SAMLResponse` form field and consume it: the request
    /// it answers is used up and the assertion ID is recorded against replay.
    ///
    /// `browser_binding` is the cookie value from the browser that posted the
    /// response (`None` when cookies are not enforced).
    pub async fn process_response(
        &self,
        saml_response_b64: &str,
        browser_binding: Option<&str>,
        enforce_binding: bool,
    ) -> Result<(ValidatedAssertion, Option<PendingRequest>), SamlError> {
        self.process_response_at(
            saml_response_b64,
            browser_binding,
            enforce_binding,
            now_secs(),
        )
        .await
    }

    /// [`Self::process_response`] at an explicit time (Unix seconds).
    pub async fn process_response_at(
        &self,
        saml_response_b64: &str,
        browser_binding: Option<&str>,
        enforce_binding: bool,
        now: i64,
    ) -> Result<(ValidatedAssertion, Option<PendingRequest>), SamlError> {
        let compact: String = saml_response_b64
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        if compact.len() > MAX_RESPONSE_BYTES * 4 / 3 + 4 {
            return fail("response too large");
        }
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(compact)
            .map_err(|_| SamlError("SAMLResponse is not base64".into()))?;
        let xml = String::from_utf8(decoded)
            .map_err(|_| SamlError("SAMLResponse is not UTF-8".into()))?;

        let mut result = self.validate_with_idp(&xml, now);
        // A response signed by a key we do not know may mean the IdP rolled its
        // certificate: refresh once (at most every five minutes) and retry.
        if matches!(&result, Err(e) if e.0.starts_with("signature:"))
            && !matches!(
                self.source,
                MetadataSource::Explicit | MetadataSource::Inline(_)
            )
            && now - *self.last_refresh.read().unwrap_or_else(|e| e.into_inner()) > 300
            && self.refresh_idp().await.is_ok()
        {
            result = self.validate_with_idp(&xml, now);
        }
        let assertion = result?;

        // Which request does this answer?
        let pending = match &assertion.in_response_to {
            Some(id) => match self.pending.get(id).map(|p| p.clone()) {
                Some(p) if now - p.created < REQUEST_TTL_SECS => {
                    if enforce_binding && browser_binding != Some(p.browser_binding.as_str()) {
                        return fail("response was not requested by this browser");
                    }
                    Some(p)
                }
                _ => return fail("response answers no pending request"),
            },
            None if self.cfg.allow_idp_initiated => None,
            None => return fail("unsolicited responses are not accepted"),
        };

        // One-time use: record the assertion ID atomically, then the request.
        self.seen.retain(|_, until| *until > now);
        use dashmap::mapref::entry::Entry;
        match self.seen.entry(assertion.assertion_id.clone()) {
            Entry::Occupied(_) => return fail("assertion was already used"),
            Entry::Vacant(v) => {
                v.insert(
                    assertion
                        .expires_at
                        .saturating_add(self.cfg.clock_skew_secs)
                        .min(now + 24 * 3600),
                );
            }
        }
        if let Some(id) = &assertion.in_response_to {
            self.pending.remove(id);
        }
        Ok((assertion, pending))
    }

    fn validate_with_idp(&self, xml: &str, now: i64) -> Result<ValidatedAssertion, SamlError> {
        let idp = self.idp();
        validate_response(
            xml,
            &ValidationContext {
                sp_entity_id: &self.cfg.sp_entity_id,
                acs_url: &self.cfg.acs_url,
                idp_entity_id: &idp.entity_id,
                idp_keys: &idp.keys,
                now,
                skew: self.cfg.clock_skew_secs,
                opts: VerifyOptions {
                    allow_sha1: self.cfg.allow_sha1,
                },
            },
        )
    }

    /// Login name, groups and profile fields of a validated assertion.
    pub fn identity(&self, a: &ValidatedAssertion) -> SamlIdentity {
        let first = |attr: &str| -> Option<String> {
            if attr.is_empty() {
                return None;
            }
            a.attributes.get(attr).and_then(|v| v.first().cloned())
        };
        let groups = if self.cfg.groups_attribute.is_empty() {
            Vec::new()
        } else {
            a.attributes
                .get(&self.cfg.groups_attribute)
                .cloned()
                .unwrap_or_default()
        };
        SamlIdentity {
            username: first(&self.cfg.username_attribute).unwrap_or_else(|| a.name_id.clone()),
            email: first(&self.cfg.email_attribute).unwrap_or_default(),
            display_name: first(&self.cfg.display_name_attribute).unwrap_or_default(),
            groups,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SamlIdentity {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub groups: Vec<String>,
}

#[cfg(test)]
mod tests;
