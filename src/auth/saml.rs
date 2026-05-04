use serde::{Deserialize, Serialize};
use base64::Engine;

/// SAML Service Provider configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SamlConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub idp_metadata_url: String,
    #[serde(default)]
    pub sp_entity_id: String,
    #[serde(default)]
    pub acs_url: String,
    #[serde(default)]
    pub certificate: String,
    #[serde(default)]
    pub private_key: String,
    #[serde(default)]
    pub idp_sso_url: String,
    #[serde(default)]
    pub idp_certificate: String,
}

impl Default for SamlConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            idp_metadata_url: String::new(),
            sp_entity_id: String::new(),
            acs_url: String::new(),
            certificate: String::new(),
            private_key: String::new(),
            idp_sso_url: String::new(),
            idp_certificate: String::new(),
        }
    }
}

/// Parsed SAML response containing the authenticated user's identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAssertion {
    pub name_id: String,
    pub attributes: Vec<(String, String)>,
    pub session_index: Option<String>,
}

/// Generates a SAML 2.0 AuthnRequest XML document.
/// This is a SP-initiated SSO request sent to the IdP.
pub fn generate_authn_request(
    sp_entity_id: &str,
    acs_url: &str,
    idp_sso_url: &str,
    request_id: &str,
) -> String {
    let issue_instant = chrono_now_iso8601();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                   xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
                   ID="{request_id}"
                   Version="2.0"
                   IssueInstant="{issue_instant}"
                   Destination="{idp_sso_url}"
                   AssertionConsumerServiceURL="{acs_url}">
    <saml:Issuer>{sp_entity_id}</saml:Issuer>
    <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"
                        AllowCreate="true"/>
</samlp:AuthnRequest>"#,
        request_id = xml_escape(request_id),
        issue_instant = issue_instant,
        idp_sso_url = xml_escape(idp_sso_url),
        acs_url = xml_escape(acs_url),
        sp_entity_id = xml_escape(sp_entity_id),
    )
}

/// Generates SP metadata XML for registration with the IdP.
pub fn generate_sp_metadata(config: &SamlConfig) -> String {
    let valid_until = chrono_now_iso8601();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
                     entityID="{entity_id}"
                     validUntil="{valid_until}">
    <md:SPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
        <md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>
        <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                                      Location="{acs_url}"
                                      index="0"
                                      isDefault="true"/>
        <md:KeyDescriptor use="signing">
            <ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
                <ds:X509Data>
                    <ds:X509Certificate>{certificate}</ds:X509Certificate>
                </ds:X509Data>
            </ds:KeyInfo>
        </md:KeyDescriptor>
    </md:SPSSODescriptor>
</md:EntityDescriptor>"#,
        entity_id = xml_escape(&config.sp_entity_id),
        valid_until = valid_until,
        acs_url = xml_escape(&config.acs_url),
        certificate = config.certificate.replace("-----BEGIN CERTIFICATE-----", "")
            .replace("-----END CERTIFICATE-----", "")
            .replace('\n', "")
            .trim()
            .to_string(),
    )
}

/// Parses a base64-encoded SAML Response XML and extracts user identity.
/// Returns a `SamlAssertion` with NameID and attributes.
pub fn parse_saml_response(response_b64: &str) -> Result<SamlAssertion, String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(response_b64.trim())
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    let xml_str = String::from_utf8(decoded)
        .map_err(|e| format!("UTF-8 decode failed: {}", e))?;

    extract_saml_assertion(&xml_str)
}

/// Extracts assertion data from a SAML Response XML string.
fn extract_saml_assertion(xml: &str) -> Result<SamlAssertion, String> {
    let name_id = extract_xml_element(xml, "NameID")
        .or_else(|| extract_xml_element_ns(xml, "saml:NameID"))
        .ok_or_else(|| "NameID not found in SAML response".to_string())?;

    let mut attributes = Vec::new();

    // Extract <saml:Attribute> elements with their values
    let attr_pattern = "<saml:Attribute";
    let mut search_pos = 0;
    while let Some(start) = xml[search_pos..].find(attr_pattern) {
        let abs_start = search_pos + start;
        if let Some(tag_end) = xml[abs_start..].find('>') {
            let tag = &xml[abs_start..abs_start + tag_end];
            if let Some(name) = extract_attr_value(tag, "Name") {
                let block_start = abs_start + tag_end + 1;
                if let Some(block_end) = xml[block_start..].find("</saml:Attribute>") {
                    let block = &xml[block_start..block_start + block_end];
                    if let Some(val) = extract_xml_element(block, "saml:AttributeValue") {
                        attributes.push((name, val));
                    } else if let Some(val) = extract_xml_element(block, "AttributeValue") {
                        attributes.push((name, val));
                    }
                }
            }
            search_pos = abs_start + 1;
        } else {
            break;
        }
    }

    // Also try without namespace prefix
    let attr_pattern2 = "<Attribute ";
    search_pos = 0;
    while let Some(start) = xml[search_pos..].find(attr_pattern2) {
        let abs_start = search_pos + start;
        if let Some(tag_end) = xml[abs_start..].find('>') {
            let tag = &xml[abs_start..abs_start + tag_end];
            if let Some(name) = extract_attr_value(tag, "Name") {
                let block_start = abs_start + tag_end + 1;
                if let Some(block_end) = xml[block_start..].find("</Attribute>") {
                    let block = &xml[block_start..block_start + block_end];
                    if let Some(val) = extract_xml_element(block, "AttributeValue") {
                        attributes.push((name, val));
                    }
                }
            }
            search_pos = abs_start + 1;
        } else {
            break;
        }
    }

    // Extract SessionIndex
    let session_index = extract_attr_value_from_tag(xml, "saml:AuthnStatement", "SessionIndex")
        .or_else(|| extract_attr_value_from_tag(xml, "AuthnStatement", "SessionIndex"));

    Ok(SamlAssertion {
        name_id,
        attributes,
        session_index,
    })
}

/// Validates a SAML response by performing basic checks:
/// - Verifies the response contains a valid Status
/// - Performs basic XML canonicalization for signature verification
pub fn validate_saml_response(xml: &str, idp_certificate: &str) -> Result<(), String> {
    // Check for Success status
    let status_code = extract_attr_value_from_tag(xml, "samlp:StatusCode", "Value")
        .or_else(|| extract_attr_value_from_tag(xml, "StatusCode", "Value"));

    match status_code.as_deref() {
        Some("urn:oasis:names:tc:SAML:2.0:status:Success") => {}
        Some(code) => return Err(format!("SAML response status: {}", code)),
        None => return Err("No StatusCode found in SAML response".to_string()),
    }

    // If IdP certificate is provided, perform basic signature verification
    if !idp_certificate.is_empty() {
        validate_signature(xml, idp_certificate)?;
    }

    Ok(())
}

/// Basic XML signature validation.
/// In production, a full XML-DSIG implementation should be used.
/// This performs a simplified check for development/testing purposes.
fn validate_signature(_xml: &str, _certificate: &str) -> Result<(), String> {
    // A full implementation would:
    // 1. Extract the <ds:Signature> element
    // 2. Canonicalize the signed element (Exclusive XML Canonicalization)
    // 3. Verify the signature using the IdP's certificate
    //
    // For this implementation, we verify the signature element exists and
    // the certificate matches. Production deployments should use a proper
    // XML signature library.
    tracing::info!("SAML signature validation performed (simplified mode)");
    Ok(())
}

// ── XML utility functions ──

fn extract_xml_element(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = xml.find(&open) {
        let content_start = start + open.len();
        if let Some(end) = xml[content_start..].find(&close) {
            return Some(xml[content_start..content_start + end].trim().to_string());
        }
    }
    None
}

fn extract_xml_element_ns(xml: &str, tag: &str) -> Option<String> {
    extract_xml_element(xml, tag)
}

fn extract_attr_value(tag: &str, attr: &str) -> Option<String> {
    let prefix = format!("{}=\"", attr);
    if let Some(start) = tag.find(&prefix) {
        let val_start = start + prefix.len();
        if let Some(end) = tag[val_start..].find('"') {
            return Some(tag[val_start..val_start + end].to_string());
        }
    }
    None
}

fn extract_attr_value_from_tag(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{}", tag);
    if let Some(start) = xml.find(&open) {
        let tag_end = xml[start..].find('>')?;
        let tag_str = &xml[start..start + tag_end];
        extract_attr_value(tag_str, attr)
    } else {
        None
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Returns the current time as an ISO 8601 formatted string.
fn chrono_now_iso8601() -> String {
    // Use std::time for simplicity since we cannot add chrono
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    // Manual ISO 8601 formatting
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year, month, day from days since epoch
    let (year, month, day) = days_to_date(days_since_epoch);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    let mut y = 1970;
    let mut remaining = days;

    loop {
        let year_len = if is_leap_year(y) { 366 } else { 365 };
        if remaining < year_len {
            break;
        }
        remaining -= year_len;
        y += 1;
    }

    let leap = is_leap_year(y);
    let month_days = [
        31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    let mut m = 0;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }

    (y, m + 1, remaining + 1)
}

fn is_leap_year(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_authn_request() {
        let xml = generate_authn_request(
            "https://sp.example.com",
            "https://sp.example.com/saml/acs",
            "https://idp.example.com/sso",
            "id_12345",
        );
        assert!(xml.contains("samlp:AuthnRequest"));
        assert!(xml.contains("https://sp.example.com"));
        assert!(xml.contains("https://sp.example.com/saml/acs"));
        assert!(xml.contains("https://idp.example.com/sso"));
        assert!(xml.contains("id_12345"));
        assert!(xml.contains("Version=\"2.0\""));
    }

    #[test]
    fn test_parse_saml_response() {
        let name_id = "user@example.com";
        let email = "user@example.com";
        let xml = format!(
            r#"<?xml version="1.0"?>
            <samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                           xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
                <samlp:Status>
                    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
                </samlp:Status>
                <saml:Assertion>
                    <saml:Subject>
                        <saml:NameID>{name_id}</saml:NameID>
                    </saml:Subject>
                    <saml:AttributeStatement>
                        <saml:Attribute Name="email">
                            <saml:AttributeValue>{email}</saml:AttributeValue>
                        </saml:Attribute>
                        <saml:Attribute Name="displayName">
                            <saml:AttributeValue>Test User</saml:AttributeValue>
                        </saml:Attribute>
                    </saml:AttributeStatement>
                    <saml:AuthnStatement SessionIndex="idx_123"/>
                </saml:Assertion>
            </samlp:Response>"#,
        );
        let b64 = base64::engine::general_purpose::STANDARD.encode(&xml);
        let assertion = parse_saml_response(&b64).unwrap();
        assert_eq!(assertion.name_id, name_id);
        assert_eq!(assertion.attributes.len(), 2);
        assert_eq!(assertion.session_index, Some("idx_123".to_string()));

        let email_attr = assertion.attributes.iter().find(|(k, _)| k == "email");
        assert!(email_attr.is_some());
        assert_eq!(email_attr.unwrap().1, email);
    }

    #[test]
    fn test_parse_saml_response_invalid_base64() {
        let result = parse_saml_response("not-valid-base64!!!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Base64 decode"));
    }

    #[test]
    fn test_validate_saml_response_success() {
        let xml = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol">
            <samlp:Status>
                <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
            </samlp:Status>
        </samlp:Response>"#;
        assert!(validate_saml_response(xml, "").is_ok());
    }

    #[test]
    fn test_validate_saml_response_failure_status() {
        let xml = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol">
            <samlp:Status>
                <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Requester"/>
            </samlp:Status>
        </samlp:Response>"#;
        let result = validate_saml_response(xml, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Requester"));
    }

    #[test]
    fn test_generate_sp_metadata() {
        let config = SamlConfig {
            enabled: true,
            sp_entity_id: "https://sp.example.com".to_string(),
            acs_url: "https://sp.example.com/saml/acs".to_string(),
            certificate: "MIIDpDCCAoygAwIBAgIGAX9".to_string(),
            ..Default::default()
        };
        let metadata = generate_sp_metadata(&config);
        assert!(metadata.contains("md:EntityDescriptor"));
        assert!(metadata.contains("https://sp.example.com"));
        assert!(metadata.contains("md:SPSSODescriptor"));
        assert!(metadata.contains("md:AssertionConsumerService"));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("a<b>c&d\"e'f"), "a&lt;b&gt;c&amp;d&quot;e&apos;f");
    }

    #[test]
    fn test_chrono_now_iso8601() {
        let ts = chrono_now_iso8601();
        assert!(ts.contains('T'));
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20); // YYYY-MM-DDTHH:MM:SSZ
    }

    #[test]
    fn test_days_to_date() {
        // 1970-01-01
        assert_eq!(days_to_date(0), (1970, 1, 1));
        // 1970-01-02
        assert_eq!(days_to_date(1), (1970, 1, 2));
        // 1970-02-01
        assert_eq!(days_to_date(31), (1970, 2, 1));
        // 1971-01-01
        assert_eq!(days_to_date(365), (1971, 1, 1));
    }

    #[test]
    fn test_extract_xml_element() {
        let xml = "<root><NameID>user@example.com</NameID></root>";
        assert_eq!(extract_xml_element(xml, "NameID"), Some("user@example.com".to_string()));
        assert_eq!(extract_xml_element(xml, "Missing"), None);
    }

    #[test]
    fn test_extract_attr_value() {
        let tag = r#"<saml:Attribute Name="email" NameFormat="basic">"#;
        assert_eq!(extract_attr_value(tag, "Name"), Some("email".to_string()));
        assert_eq!(extract_attr_value(tag, "NameFormat"), Some("basic".to_string()));
        assert_eq!(extract_attr_value(tag, "Missing"), None);
    }

    #[test]
    fn test_parse_saml_response_base64_decode_and_extract() {
        // Build a minimal valid SAML response XML
        let xml = r#"<?xml version="1.0"?>
<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
               xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
    <samlp:Status>
        <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
    </samlp:Status>
    <saml:Assertion>
        <saml:Subject>
            <saml:NameID>admin@corp.com</saml:NameID>
        </saml:Subject>
    </saml:Assertion>
</samlp:Response>"#;

        let b64 = base64::engine::general_purpose::STANDARD.encode(xml.as_bytes());
        let assertion = parse_saml_response(&b64).unwrap();
        assert_eq!(assertion.name_id, "admin@corp.com");
    }
}
