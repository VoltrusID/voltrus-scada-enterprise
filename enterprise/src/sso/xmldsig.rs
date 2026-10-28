//! XML Signature (XMLDSig) verification for SAML, in pure Rust.
//!
//! Implements exactly the profile SAML 2.0 uses: an enveloped `ds:Signature`
//! whose single `ds:Reference` points at the signed element by `ID`, with the
//! `enveloped-signature` and `exc-c14n` transforms, exclusive canonicalization
//! of `SignedInfo`, and an RSA PKCS#1 v1.5 signature. Everything else is
//! refused rather than guessed at.
//!
//! The signed element is never found by looking up the `URI` in the document.
//! It is the parent of the `Signature` element, and the `URI` must name that
//! parent's own `ID`. A signature can therefore only ever vouch for the element
//! it is physically attached to, which closes XML signature wrapping.

use roxmltree::{Document, Node, NodeId};
use rsa::RsaPublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::sha2::{Digest, Sha256, Sha384, Sha512};
use rsa::signature::hazmat::PrehashVerifier;
use std::collections::BTreeMap;

pub const NS_DS: &str = "http://www.w3.org/2000/09/xmldsig#";
const NS_XML: &str = "http://www.w3.org/XML/1998/namespace";
const NS_EXC_C14N: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";

const ALG_EXC_C14N: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";
const ALG_ENVELOPED: &str = "http://www.w3.org/2000/09/xmldsig#enveloped-signature";

const SIG_RSA_SHA1: &str = "http://www.w3.org/2000/09/xmldsig#rsa-sha1";
const SIG_RSA_SHA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
const SIG_RSA_SHA384: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha384";
const SIG_RSA_SHA512: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha512";

const DIG_SHA1: &str = "http://www.w3.org/2000/09/xmldsig#sha1";
const DIG_SHA256: &str = "http://www.w3.org/2001/04/xmlenc#sha256";
const DIG_SHA384: &str = "http://www.w3.org/2001/04/xmldsig-more#sha384";
const DIG_SHA512: &str = "http://www.w3.org/2001/04/xmlenc#sha512";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hash {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl Hash {
    fn digest(self, data: &[u8]) -> Vec<u8> {
        match self {
            Hash::Sha1 => sha1::Sha1::digest(data).to_vec(),
            Hash::Sha256 => Sha256::digest(data).to_vec(),
            Hash::Sha384 => Sha384::digest(data).to_vec(),
            Hash::Sha512 => Sha512::digest(data).to_vec(),
        }
    }
}

/// Why a signature was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigError(pub String);

impl std::fmt::Display for SigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SigError {}

fn err<T>(msg: impl Into<String>) -> Result<T, SigError> {
    Err(SigError(msg.into()))
}

/// Algorithm policy for verification.
#[derive(Debug, Clone, Copy, Default)]
pub struct VerifyOptions {
    /// Accept RSA-SHA1 signatures and SHA-1 digests. Off unless the operator
    /// opts in for an IdP that cannot do better.
    pub allow_sha1: bool,
}

// ---------------------------------------------------------------------------
// Tree helpers
// ---------------------------------------------------------------------------

/// True when `node` is an element with the given namespace and local name.
pub fn is_element(node: Node, ns: &str, local: &str) -> bool {
    node.is_element() && node.tag_name().namespace() == Some(ns) && node.tag_name().name() == local
}

/// Child elements with the given namespace and local name.
pub fn children<'a, 'i>(
    node: Node<'a, 'i>,
    ns: &'static str,
    local: &'static str,
) -> impl Iterator<Item = Node<'a, 'i>> {
    node.children().filter(move |c| is_element(*c, ns, local))
}

/// The only child with that name; `Err` if there are none or several.
fn single_child<'a, 'i>(
    node: Node<'a, 'i>,
    ns: &'static str,
    local: &'static str,
) -> Result<Node<'a, 'i>, SigError> {
    let mut it = children(node, ns, local);
    match (it.next(), it.next()) {
        (Some(c), None) => Ok(c),
        (None, _) => err(format!("missing <{local}>")),
        (Some(_), Some(_)) => err(format!("duplicate <{local}>")),
    }
}

fn algorithm<'a>(node: Node<'a, '_>) -> Result<&'a str, SigError> {
    node.attribute("Algorithm")
        .ok_or_else(|| SigError(format!("<{}> has no Algorithm", node.tag_name().name())))
}

/// Concatenated text of an element, whitespace removed (base64 payloads).
fn compact_text(node: Node) -> String {
    node.descendants()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .flat_map(|t| t.chars())
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// Reject documents where two elements share an `ID`. Duplicate IDs are the
/// raw material of signature wrapping attacks.
pub fn ensure_unique_ids(doc: &Document) -> Result<(), SigError> {
    let mut seen = std::collections::HashSet::new();
    for n in doc.descendants().filter(|n| n.is_element()) {
        for a in n.attributes() {
            if a.namespace().is_none()
                && (a.name() == "ID" || a.name() == "Id" || a.name() == "id")
                && !seen.insert(a.value().to_string())
            {
                return err(format!("duplicate ID '{}'", a.value()));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Exclusive canonicalization (http://www.w3.org/2001/10/xml-exc-c14n#)
// ---------------------------------------------------------------------------

struct Canonicalizer<'s> {
    src: &'s str,
    exclude: Option<NodeId>,
    inclusive_prefixes: Vec<String>,
    out: String,
}

/// Canonicalize the subtree at `root` with exclusive C14N (no comments).
///
/// `exclude` drops one element (the enveloped `Signature`). `inclusive_prefixes`
/// is the `InclusiveNamespaces` `PrefixList` (`#default` for the default
/// namespace).
pub fn canonicalize(
    src: &str,
    root: Node,
    exclude: Option<NodeId>,
    inclusive_prefixes: &[String],
) -> String {
    let mut c = Canonicalizer {
        src,
        exclude,
        inclusive_prefixes: inclusive_prefixes
            .iter()
            .map(|p| {
                if p == "#default" {
                    String::new()
                } else {
                    p.clone()
                }
            })
            .collect(),
        out: String::new(),
    };
    c.element(root, &BTreeMap::new());
    c.out
}

/// The qualified name of an element exactly as written in the source.
fn source_qname<'s>(src: &'s str, start: usize) -> &'s str {
    let rest = src.get(start..).unwrap_or("");
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(rest.len());
    &rest[..end]
}

fn split_prefix(qname: &str) -> (&str, &str) {
    match qname.split_once(':') {
        Some((p, l)) => (p, l),
        None => ("", qname),
    }
}

/// Namespace URI bound to `prefix` ("" for the default namespace) at `node`.
fn lookup_ns<'a>(node: Node<'a, '_>, prefix: &str) -> Option<&'a str> {
    if prefix == "xml" {
        return Some(NS_XML);
    }
    for anc in node.ancestors() {
        for ns in anc.namespaces() {
            if ns.name().unwrap_or("") == prefix {
                return Some(ns.uri());
            }
        }
    }
    None
}

fn escape_text(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\r' => out.push_str("&#xD;"),
            c => out.push(c),
        }
    }
}

fn escape_attr(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            '\t' => out.push_str("&#x9;"),
            '\n' => out.push_str("&#xA;"),
            '\r' => out.push_str("&#xD;"),
            c => out.push(c),
        }
    }
}

impl Canonicalizer<'_> {
    fn element(&mut self, node: Node, rendered: &BTreeMap<String, String>) {
        let qname = source_qname(self.src, node.range().start + 1);
        let (el_prefix, _) = split_prefix(qname);

        // Attributes with their source qname, sorted by (namespace URI, local name).
        struct Attr<'a> {
            ns: &'a str,
            local: &'a str,
            qname: &'a str,
            value: &'a str,
        }
        let mut attrs: Vec<Attr> = node
            .attributes()
            .map(|a| Attr {
                ns: a.namespace().unwrap_or(""),
                local: a.name(),
                // `range_qname` saturates for names over 64 KiB; fall back to the local name.
                qname: self.src.get(a.range_qname()).unwrap_or(a.name()),
                value: a.value(),
            })
            .collect();
        attrs.sort_by(|a, b| (a.ns, a.local).cmp(&(b.ns, b.local)));

        // Namespaces visibly utilized by this element and its attributes.
        let mut utilized: Vec<String> = vec![el_prefix.to_string()];
        for a in &attrs {
            let (p, _) = split_prefix(a.qname);
            if !p.is_empty() {
                utilized.push(p.to_string());
            }
        }
        for p in &self.inclusive_prefixes {
            utilized.push(p.clone());
        }
        utilized.sort();
        utilized.dedup();

        let mut new_rendered = rendered.clone();
        let mut decls: Vec<(String, String)> = Vec::new();
        for p in utilized {
            if p == "xml" {
                continue;
            }
            let uri = lookup_ns(node, &p).unwrap_or("");
            let already = rendered.get(&p).map(String::as_str);
            let emit = if uri.is_empty() {
                // `xmlns=""` only to undeclare a default namespace we rendered.
                p.is_empty() && already.is_some_and(|u| !u.is_empty())
            } else {
                already != Some(uri)
            };
            if emit {
                new_rendered.insert(p.clone(), uri.to_string());
                decls.push((p, uri.to_string()));
            }
        }
        decls.sort();

        self.out.push('<');
        self.out.push_str(qname);
        for (p, uri) in &decls {
            if p.is_empty() {
                self.out.push_str(" xmlns=\"");
            } else {
                self.out.push_str(" xmlns:");
                self.out.push_str(p);
                self.out.push_str("=\"");
            }
            escape_attr(uri, &mut self.out);
            self.out.push('"');
        }
        for a in &attrs {
            self.out.push(' ');
            self.out.push_str(a.qname);
            self.out.push_str("=\"");
            escape_attr(a.value, &mut self.out);
            self.out.push('"');
        }
        self.out.push('>');

        for child in node.children() {
            if Some(child.id()) == self.exclude {
                continue;
            }
            if child.is_element() {
                self.element(child, &new_rendered);
            } else if child.is_text() {
                escape_text(child.text().unwrap_or(""), &mut self.out);
            } else if child.is_pi()
                && let Some(pi) = child.pi()
            {
                self.out.push_str("<?");
                self.out.push_str(pi.target);
                if let Some(v) = pi.value
                    && !v.is_empty()
                {
                    self.out.push(' ');
                    self.out.push_str(v);
                }
                self.out.push_str("?>");
            }
            // Comments are dropped (canonicalization without comments).
        }

        self.out.push_str("</");
        self.out.push_str(qname);
        self.out.push('>');
    }
}

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

fn inclusive_prefixes(method: Node) -> Result<Vec<String>, SigError> {
    let mut prefixes = Vec::new();
    for ch in method.children().filter(|c| c.is_element()) {
        if is_element(ch, NS_EXC_C14N, "InclusiveNamespaces") {
            let list = ch.attribute("PrefixList").unwrap_or("");
            prefixes.extend(list.split_whitespace().map(str::to_string));
        } else {
            return err(format!(
                "unexpected <{}> in canonicalization method",
                ch.tag_name().name()
            ));
        }
    }
    Ok(prefixes)
}

fn digest_hash(alg: &str, opts: VerifyOptions) -> Result<Hash, SigError> {
    match alg {
        DIG_SHA256 => Ok(Hash::Sha256),
        DIG_SHA384 => Ok(Hash::Sha384),
        DIG_SHA512 => Ok(Hash::Sha512),
        DIG_SHA1 if opts.allow_sha1 => Ok(Hash::Sha1),
        DIG_SHA1 => err("SHA-1 digests are not allowed"),
        other => err(format!("unsupported digest algorithm {other}")),
    }
}

fn signature_hash(alg: &str, opts: VerifyOptions) -> Result<Hash, SigError> {
    match alg {
        SIG_RSA_SHA256 => Ok(Hash::Sha256),
        SIG_RSA_SHA384 => Ok(Hash::Sha384),
        SIG_RSA_SHA512 => Ok(Hash::Sha512),
        SIG_RSA_SHA1 if opts.allow_sha1 => Ok(Hash::Sha1),
        SIG_RSA_SHA1 => err("RSA-SHA1 signatures are not allowed"),
        other => err(format!("unsupported signature algorithm {other}")),
    }
}

fn verify_rsa(
    key: &RsaPublicKey,
    hash: Hash,
    prehash: &[u8],
    signature: &[u8],
) -> Result<(), SigError> {
    let sig = Signature::try_from(signature).map_err(|_| SigError("malformed signature".into()))?;
    let res = match hash {
        Hash::Sha1 => VerifyingKey::<sha1::Sha1>::new(key.clone()).verify_prehash(prehash, &sig),
        Hash::Sha256 => VerifyingKey::<Sha256>::new(key.clone()).verify_prehash(prehash, &sig),
        Hash::Sha384 => VerifyingKey::<Sha384>::new(key.clone()).verify_prehash(prehash, &sig),
        Hash::Sha512 => VerifyingKey::<Sha512>::new(key.clone()).verify_prehash(prehash, &sig),
    };
    res.map_err(|_| SigError("signature value does not match".into()))
}

/// Verify one enveloped `ds:Signature` and return the element it signs.
///
/// `sig` must be a `ds:Signature` element. Its parent is the signed element;
/// the caller decides which parents are acceptable. `keys` are the trusted
/// public keys (pinned from IdP metadata); any key inside the signature's own
/// `KeyInfo` is ignored. Pass the result, not the document, to the code that
/// reads claims.
pub fn verify_enveloped_signature<'a, 'i>(
    src: &str,
    sig: Node<'a, 'i>,
    keys: &[RsaPublicKey],
    opts: VerifyOptions,
) -> Result<Node<'a, 'i>, SigError> {
    if !is_element(sig, NS_DS, "Signature") {
        return err("not a ds:Signature element");
    }
    if keys.is_empty() {
        return err("no trusted signing key");
    }
    let signed = sig
        .parent()
        .filter(|p| p.is_element())
        .ok_or_else(|| SigError("Signature has no parent element".into()))?;
    let signed_id = signed
        .attribute("ID")
        .ok_or_else(|| SigError("signed element has no ID".into()))?;

    let signed_info = single_child(sig, NS_DS, "SignedInfo")?;
    let sig_value = single_child(sig, NS_DS, "SignatureValue")?;

    let c14n_method = single_child(signed_info, NS_DS, "CanonicalizationMethod")?;
    if algorithm(c14n_method)? != ALG_EXC_C14N {
        return err("only exclusive canonicalization without comments is supported");
    }
    let signed_info_prefixes = inclusive_prefixes(c14n_method)?;
    let sig_hash = signature_hash(
        algorithm(single_child(signed_info, NS_DS, "SignatureMethod")?)?,
        opts,
    )?;

    // Exactly one Reference, and it must point at the element carrying the signature.
    let reference = single_child(signed_info, NS_DS, "Reference")?;
    let uri = reference.attribute("URI").unwrap_or("");
    if uri.strip_prefix('#') != Some(signed_id) {
        return err("Reference URI does not name the signed element");
    }
    let mut ref_prefixes = Vec::new();
    let mut enveloped = false;
    let mut c14n_seen = false;
    if let Some(transforms) = children(reference, NS_DS, "Transforms").next() {
        for t in children(transforms, NS_DS, "Transform") {
            match algorithm(t)? {
                ALG_ENVELOPED if !enveloped && !c14n_seen => enveloped = true,
                ALG_EXC_C14N if !c14n_seen => {
                    c14n_seen = true;
                    ref_prefixes = inclusive_prefixes(t)?;
                }
                other => return err(format!("unsupported transform {other}")),
            }
        }
    }
    if !enveloped {
        return err("Reference lacks the enveloped-signature transform");
    }
    if !c14n_seen {
        return err("Reference lacks the exclusive canonicalization transform");
    }
    let dig_hash = digest_hash(
        algorithm(single_child(reference, NS_DS, "DigestMethod")?)?,
        opts,
    )?;
    let digest_value = base64_decode(&compact_text(single_child(
        reference,
        NS_DS,
        "DigestValue",
    )?))?;

    // 1. The digest of the signed element (minus this Signature) must match.
    let canonical = canonicalize(src, signed, Some(sig.id()), &ref_prefixes);
    let actual = dig_hash.digest(canonical.as_bytes());
    if !constant_time_eq(&actual, &digest_value) {
        return err("digest of the signed element does not match");
    }

    // 2. The signature over canonical SignedInfo must verify with a pinned key.
    let signed_info_c14n = canonicalize(src, signed_info, None, &signed_info_prefixes);
    let prehash = sig_hash.digest(signed_info_c14n.as_bytes());
    let signature = base64_decode(&compact_text(sig_value))?;
    let mut last = SigError("no trusted signing key".into());
    for key in keys {
        match verify_rsa(key, sig_hash, &prehash, &signature) {
            Ok(()) => return Ok(signed),
            Err(e) => last = e,
        }
    }
    Err(last)
}

fn base64_decode(s: &str) -> Result<Vec<u8>, SigError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|_| SigError("invalid base64".into()))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Parse an X.509 certificate (PEM or bare base64 DER) and return its RSA public key.
pub fn rsa_key_from_certificate(cert: &str) -> Result<RsaPublicKey, SigError> {
    use rsa::pkcs8::DecodePublicKey;
    use x509_cert::Certificate;
    use x509_cert::der::{Decode, Encode};

    let b64: String = cert
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("-----"))
        .collect();
    let der = base64_decode(&b64)?;
    let cert =
        Certificate::from_der(&der).map_err(|e| SigError(format!("bad certificate: {e}")))?;
    let spki = cert
        .tbs_certificate
        .subject_public_key_info
        .to_der()
        .map_err(|e| SigError(format!("bad certificate key: {e}")))?;
    let key = RsaPublicKey::from_public_key_der(&spki)
        .map_err(|_| SigError("certificate does not hold an RSA public key".into()))?;
    rsa_public_key_checked(&key)
}

/// Refuse keys too small to be a trust anchor.
pub fn rsa_public_key_checked(key: &RsaPublicKey) -> Result<RsaPublicKey, SigError> {
    use rsa::traits::PublicKeyParts;
    if key.size() < 256 {
        return err(format!(
            "RSA key of {} bits is below the 2048 bit minimum",
            key.size() * 8
        ));
    }
    Ok(key.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c14n_of(
        xml: &str,
        pick: impl Fn(&Document) -> roxmltree::NodeId,
        prefixes: &[&str],
    ) -> String {
        let doc = Document::parse(xml).unwrap();
        let id = pick(&doc);
        let prefixes: Vec<String> = prefixes.iter().map(|s| s.to_string()).collect();
        canonicalize(xml, doc.get_node(id).unwrap(), None, &prefixes)
    }

    // Expected values below were produced by libxml2: `xmllint --exc-c14n`.

    #[test]
    fn exclusive_c14n_matches_libxml2_on_namespaces_attributes_and_escaping() {
        let xml = "<a:root xmlns:a=\"urn:a\" xmlns:b=\"urn:b\" xmlns=\"urn:d\" z=\"1\" a:y=\"2\" b:x='3&amp;\"'><child xmlns=\"\">t&lt;e&#13;xt &gt;&amp;<!-- c --></child><b:leaf xmlns:b=\"urn:b\" xmlns:unused=\"urn:u\"/><inner a:k=\"v\"><?pi data?></inner><e2 xmlns=\"urn:e\"><x/></e2></a:root>";
        let got = c14n_of(xml, |d| d.root_element().id(), &[]);
        assert_eq!(
            got,
            "<a:root xmlns:a=\"urn:a\" xmlns:b=\"urn:b\" z=\"1\" a:y=\"2\" b:x=\"3&amp;&quot;\"><child>t&lt;e&#xD;xt &gt;&amp;</child><b:leaf></b:leaf><inner xmlns=\"urn:d\" a:k=\"v\"><?pi data?></inner><e2 xmlns=\"urn:e\"><x></x></e2></a:root>"
        );
    }

    #[test]
    fn a_subtree_declares_only_the_namespaces_it_uses_and_inherits_no_xml_attributes() {
        let xml = "<root xmlns:p=\"urn:p\" xmlns:q=\"urn:q\" xml:lang=\"en\"><p:a q:attr=\"1\"><p:b/></p:a></root>";
        let got = c14n_of(
            xml,
            |d| d.root_element().first_element_child().unwrap().id(),
            &[],
        );
        assert_eq!(
            got,
            "<p:a xmlns:p=\"urn:p\" xmlns:q=\"urn:q\" q:attr=\"1\"><p:b></p:b></p:a>"
        );
    }

    #[test]
    fn inclusive_prefix_list_forces_unused_declarations() {
        let xml = "<root xmlns:p=\"urn:p\" xmlns:xs=\"urn:xs\"><p:a/></root>";
        let got = c14n_of(xml, |d| d.root_element().id(), &["xs"]);
        assert_eq!(
            got,
            "<root xmlns:xs=\"urn:xs\"><p:a xmlns:p=\"urn:p\"></p:a></root>"
        );
    }

    #[test]
    fn the_enveloped_signature_is_left_out() {
        let xml = "<r xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\"><a/><ds:Signature><ds:x/></ds:Signature><b/></r>";
        let doc = Document::parse(xml).unwrap();
        let sig = doc
            .descendants()
            .find(|n| n.tag_name().name() == "Signature")
            .unwrap()
            .id();
        assert_eq!(
            canonicalize(xml, doc.root_element(), Some(sig), &[]),
            "<r><a></a><b></b></r>"
        );
    }
}
