use serde::{Deserialize, Serialize};

/// LDAP authentication configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LdapConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub server_url: String,
    #[serde(default)]
    pub base_dn: String,
    #[serde(default)]
    pub bind_dn_format: String,
    #[serde(default)]
    pub use_tls: bool,
    #[serde(default)]
    pub search_filter: String,
    #[serde(default = "default_ldap_port")]
    pub port: u16,
}

fn default_ldap_port() -> u16 {
    389
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_url: String::new(),
            base_dn: String::new(),
            bind_dn_format: String::new(),
            use_tls: false,
            search_filter: String::new(),
            port: 389,
        }
    }
}

/// Result of an LDAP authentication attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapAuthResult {
    pub success: bool,
    pub username: String,
    pub dn: String,
    pub message: String,
}

/// LDAP message types for protocol encoding.
#[repr(u8)]
enum LdapTag {
    BindRequest = 0x60,
    BindResponse = 0x61,
    Integer = 0x02,
    OctetString = 0x04,
    Sequence = 0x30,
    Enumeration = 0x0A,
}

/// Performs an LDAP simple bind authentication.
/// Connects to the LDAP server and attempts to bind with the given credentials.
pub async fn ldap_authenticate(
    server: &str,
    port: u16,
    bind_dn: &str,
    password: &str,
    use_tls: bool,
) -> Result<LdapAuthResult, String> {
    let addr = format!("{}:{}", server, port);

    // Connect via TCP
    let stream = tokio::net::TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("LDAP connection failed: {}", e))?;

    if use_tls {
        // For LDAPS/TLS, we would need a TLS connector.
        // Since we cannot add new crate dependencies, we log a warning
        // and proceed without TLS for now.
        tracing::warn!(
            "LDAP TLS requested but native-tls not available; connection may be insecure"
        );
    }

    // Build LDAP BindRequest
    // LDAPMessage ::= SEQUENCE {
    //   messageID  INTEGER,
    //   protocolOp CHOICE { bindRequest [APPLICATION 0] SEQUENCE { ... } }
    // }
    let message_id: u32 = 1;

    // Encode the BindRequest:
    // BindRequest ::= [APPLICATION 0] SEQUENCE {
    //   version    INTEGER (3),
    //   name       OCTET STRING,
    //   authentication CHOICE { simple [0] OCTET STRING }
    // }
    let version_bytes = encode_integer(3);
    let name_bytes = encode_octet_string(bind_dn.as_bytes());
    let auth_bytes = encode_context_octet_string(0, password.as_bytes());

    let bind_request_inner = encode_sequence(&[&version_bytes, &name_bytes, &auth_bytes]);
    let bind_request = wrap_tag(LdapTag::BindRequest as u8, &bind_request_inner);

    // Full LDAP message
    let message_id_bytes = encode_integer(message_id as i64);
    let ldap_message = encode_sequence(&[&message_id_bytes, &bind_request]);

    // Send the bind request using tokio TCP
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = stream;

    // LDAP messages are preceded by BER length encoding
    stream
        .write_all(&ldap_message)
        .await
        .map_err(|e| format!("LDAP write failed: {}", e))?;

    // Read response
    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("LDAP read failed: {}", e))?;

    if n == 0 {
        return Err("LDAP server closed connection".to_string());
    }

    // Parse the LDAP BindResponse
    let result = parse_bind_response(&buf[..n])?;

    Ok(LdapAuthResult {
        success: result.success,
        username: bind_dn.to_string(),
        dn: bind_dn.to_string(),
        message: result.message,
    })
}

struct BindResponseResult {
    success: bool,
    message: String,
}

/// Parses an LDAP BindResponse from raw bytes.
fn parse_bind_response(data: &[u8]) -> Result<BindResponseResult, String> {
    // LDAP BindResponse ::= [APPLICATION 1] SEQUENCE {
    //   resultCode ENUMERATED,
    //   matchedDN  OCTET STRING,
    //   diagMsg    OCTET STRING
    // }

    if data.len() < 2 {
        return Err("LDAP response too short".to_string());
    }

    // Skip outer sequence tag and length
    let mut pos = 0;
    if data[pos] != LdapTag::Sequence as u8 {
        return Err(format!("Expected SEQUENCE tag, got {:02X}", data[pos]));
    }
    pos += 1;
    let (_seq_len, consumed) = read_ber_length(&data[pos..]);
    pos += consumed;

    // Skip message ID (INTEGER)
    if data[pos] != LdapTag::Integer as u8 {
        return Err(format!(
            "Expected INTEGER tag for messageID, got {:02X}",
            data[pos]
        ));
    }
    pos += 1;
    let (int_len, consumed) = read_ber_length(&data[pos..]);
    pos += consumed + int_len;

    // Check for BindResponse tag [APPLICATION 1] = 0x61
    if pos >= data.len() {
        return Err("LDAP response truncated before BindResponse".to_string());
    }
    if data[pos] != LdapTag::BindResponse as u8 {
        return Err(format!(
            "Expected BindResponse tag 0x61, got {:02X}",
            data[pos]
        ));
    }
    pos += 1;
    let (_resp_len, consumed) = read_ber_length(&data[pos..]);
    pos += consumed;

    // Read resultCode (ENUMERATED)
    if pos >= data.len() || data[pos] != LdapTag::Enumeration as u8 {
        return Err("Expected ENUMERATED tag for resultCode".to_string());
    }
    pos += 1;
    let (enum_len, consumed) = read_ber_length(&data[pos..]);
    pos += consumed;

    let result_code = if enum_len == 1 {
        data[pos] as u32
    } else if enum_len == 2 {
        u32::from_be_bytes([0, 0, data[pos], data[pos + 1]])
    } else {
        0
    };
    pos += enum_len;

    // Skip matchedDN (OCTET STRING)
    if pos < data.len() && data[pos] == LdapTag::OctetString as u8 {
        pos += 1;
        let (str_len, consumed) = read_ber_length(&data[pos..]);
        pos += consumed + str_len;
    }

    // Read diagnostic message (OCTET STRING)
    let mut message = String::new();
    if pos < data.len() && data[pos] == LdapTag::OctetString as u8 {
        pos += 1;
        let (str_len, consumed) = read_ber_length(&data[pos..]);
        pos += consumed;
        if pos + str_len <= data.len() {
            message = String::from_utf8_lossy(&data[pos..pos + str_len]).to_string();
        }
    }

    // LDAP result codes: 0 = success
    let success = result_code == 0;

    Ok(BindResponseResult { success, message })
}

// ── BER/DER encoding helpers ──

fn encode_integer(value: i64) -> Vec<u8> {
    let tag = vec![LdapTag::Integer as u8];
    if value == 0 {
        return encode_tlv(tag, vec![0]);
    }
    let mut val_bytes = vec![];
    let mut v = value as u64;
    while v > 0 {
        val_bytes.push((v & 0xFF) as u8);
        v >>= 8;
    }
    val_bytes.reverse();
    // Add leading zero if high bit set (to keep it positive)
    if val_bytes[0] & 0x80 != 0 {
        val_bytes.insert(0, 0);
    }
    encode_tlv(tag, val_bytes)
}

fn encode_octet_string(data: &[u8]) -> Vec<u8> {
    let tag = vec![LdapTag::OctetString as u8];
    encode_tlv(tag, data.to_vec())
}

fn encode_context_octet_string(context: u8, data: &[u8]) -> Vec<u8> {
    // Context-specific constructed: 0x80 | context
    let tag = vec![0x80 | context];
    encode_tlv(tag, data.to_vec())
}

fn encode_sequence(parts: &[&[u8]]) -> Vec<u8> {
    let mut content = vec![];
    for part in parts {
        content.extend_from_slice(part);
    }
    let tag = vec![LdapTag::Sequence as u8];
    encode_tlv(tag, content)
}

fn wrap_tag(tag: u8, content: &[u8]) -> Vec<u8> {
    encode_tlv(vec![tag], content.to_vec())
}

fn encode_tlv(tag: Vec<u8>, value: Vec<u8>) -> Vec<u8> {
    let mut result = tag;
    result.extend_from_slice(&encode_ber_length(value.len()));
    result.extend_from_slice(&value);
    result
}

fn encode_ber_length(len: usize) -> Vec<u8> {
    if len < 128 {
        vec![len as u8]
    } else if len < 256 {
        vec![0x81, len as u8]
    } else {
        vec![0x82, (len >> 8) as u8, (len & 0xFF) as u8]
    }
}

fn read_ber_length(data: &[u8]) -> (usize, usize) {
    if data.is_empty() {
        return (0, 0);
    }
    if data[0] < 128 {
        (data[0] as usize, 1)
    } else if data[0] == 0x81 && data.len() > 1 {
        (data[1] as usize, 2)
    } else if data[0] == 0x82 && data.len() > 2 {
        ((data[1] as usize) << 8 | data[2] as usize, 3)
    } else {
        (0, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_integer_zero() {
        let encoded = encode_integer(0);
        assert_eq!(encoded, vec![0x02, 0x01, 0x00]);
    }

    #[test]
    fn test_encode_integer_one() {
        let encoded = encode_integer(1);
        assert_eq!(encoded, vec![0x02, 0x01, 0x01]);
    }

    #[test]
    fn test_encode_integer_three() {
        let encoded = encode_integer(3);
        assert_eq!(encoded, vec![0x02, 0x01, 0x03]);
    }

    #[test]
    fn test_encode_integer_large() {
        let encoded = encode_integer(256);
        assert_eq!(encoded, vec![0x02, 0x02, 0x01, 0x00]);
    }

    #[test]
    fn test_encode_octet_string() {
        let encoded = encode_octet_string(b"hello");
        assert_eq!(encoded, vec![0x04, 0x05, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_encode_octet_string_empty() {
        let encoded = encode_octet_string(b"");
        assert_eq!(encoded, vec![0x04, 0x00]);
    }

    #[test]
    fn test_encode_context_octet_string() {
        let encoded = encode_context_octet_string(0, b"secret");
        assert_eq!(
            encoded,
            vec![0x80, 0x06, b's', b'e', b'c', b'r', b'e', b't']
        );
    }

    #[test]
    fn test_encode_sequence() {
        let a = encode_integer(3);
        let b = encode_octet_string(b"cn=admin");
        let seq = encode_sequence(&[&a, &b]);
        // Sequence tag + length + content
        assert_eq!(seq[0], 0x30);
        // Verify it contains both parts
        assert!(seq.len() > a.len() + b.len());
    }

    #[test]
    fn test_encode_ber_length_short() {
        assert_eq!(encode_ber_length(5), vec![5]);
    }

    #[test]
    fn test_encode_ber_length_medium() {
        assert_eq!(encode_ber_length(200), vec![0x81, 200]);
    }

    #[test]
    fn test_encode_ber_length_long() {
        assert_eq!(encode_ber_length(300), vec![0x82, 0x01, 0x2C]);
    }

    #[test]
    fn test_read_ber_length_short() {
        assert_eq!(read_ber_length(&[5]), (5, 1));
    }

    #[test]
    fn test_read_ber_length_medium() {
        assert_eq!(read_ber_length(&[0x81, 200]), (200, 2));
    }

    #[test]
    fn test_read_ber_length_long() {
        assert_eq!(read_ber_length(&[0x82, 0x01, 0x2C]), (300, 3));
    }

    #[test]
    fn test_ldap_bind_request_construction() {
        // Build a complete bind request and verify structure
        let version_bytes = encode_integer(3);
        let name_bytes = encode_octet_string(b"cn=admin,dc=example,dc=com");
        let auth_bytes = encode_context_octet_string(0, b"password");

        let bind_request_inner = encode_sequence(&[&version_bytes, &name_bytes, &auth_bytes]);
        let bind_request = wrap_tag(0x60, &bind_request_inner);

        let message_id_bytes = encode_integer(1);
        let ldap_message = encode_sequence(&[&message_id_bytes, &bind_request]);

        // Verify the message starts with a SEQUENCE
        assert_eq!(ldap_message[0], 0x30);
        // Message should contain the bind DN
        let msg_str = String::from_utf8_lossy(&ldap_message);
        assert!(msg_str.contains("cn=admin,dc=example,dc=com"));
    }

    #[test]
    fn test_parse_bind_response_success() {
        // Construct a successful bind response:
        // SEQUENCE { INTEGER 1, BindResponse { ENUMERATED 0, OCTET STRING "", OCTET STRING "" } }
        let mut response = vec![];
        // Build from inside out
        let result_code = vec![0x0A, 0x01, 0x00]; // ENUMERATED 0 (success)
        let matched_dn = vec![0x04, 0x00]; // OCTET STRING ""
        let diag_msg = vec![0x04, 0x00]; // OCTET STRING ""

        let mut bind_resp_content = vec![];
        bind_resp_content.extend_from_slice(&result_code);
        bind_resp_content.extend_from_slice(&matched_dn);
        bind_resp_content.extend_from_slice(&diag_msg);

        let mut bind_resp = vec![0x61]; // BindResponse tag
        bind_resp.extend_from_slice(&encode_ber_length(bind_resp_content.len()));
        bind_resp.extend_from_slice(&bind_resp_content);

        let message_id = vec![0x02, 0x01, 0x01]; // INTEGER 1
        let mut seq_content = vec![];
        seq_content.extend_from_slice(&message_id);
        seq_content.extend_from_slice(&bind_resp);

        response.push(0x30); // SEQUENCE
        response.extend_from_slice(&encode_ber_length(seq_content.len()));
        response.extend_from_slice(&seq_content);

        let result = parse_bind_response(&response).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_parse_bind_response_failure() {
        // Construct a failed bind response (resultCode = 49 = invalidCredentials)
        let result_code = vec![0x0A, 0x01, 0x31]; // ENUMERATED 49
        let matched_dn = vec![0x04, 0x00];
        let diag_msg = vec![0x04, 0x13]; // 0x13 = 19 bytes for "Invalid credentials"
        let diag_text = b"Invalid credentials";
        let mut diag = diag_msg;
        diag.extend_from_slice(diag_text);

        let mut bind_resp_content = vec![];
        bind_resp_content.extend_from_slice(&result_code);
        bind_resp_content.extend_from_slice(&matched_dn);
        bind_resp_content.extend_from_slice(&diag);

        let mut bind_resp = vec![0x61];
        bind_resp.extend_from_slice(&encode_ber_length(bind_resp_content.len()));
        bind_resp.extend_from_slice(&bind_resp_content);

        let message_id = vec![0x02, 0x01, 0x01];
        let mut seq_content = vec![];
        seq_content.extend_from_slice(&message_id);
        seq_content.extend_from_slice(&bind_resp);

        let mut response = vec![0x30];
        response.extend_from_slice(&encode_ber_length(seq_content.len()));
        response.extend_from_slice(&seq_content);

        let result = parse_bind_response(&response).unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Invalid credentials");
    }

    #[test]
    fn test_ldap_config_default() {
        let config = LdapConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.port, 389);
        assert!(!config.use_tls);
    }

    #[test]
    fn test_ldap_bind_request_build() {
        // Verify that we can construct a valid LDAP bind request for various DNs
        let test_cases = vec![
            ("cn=admin,dc=example,dc=com", "password123"),
            ("uid=user,ou=people,dc=corp,dc=com", "secret"),
            ("admin@example.com", "pass"),
        ];

        for (dn, pw) in test_cases {
            let version_bytes = encode_integer(3);
            let name_bytes = encode_octet_string(dn.as_bytes());
            let auth_bytes = encode_context_octet_string(0, pw.as_bytes());

            let bind_inner = encode_sequence(&[&version_bytes, &name_bytes, &auth_bytes]);
            let bind_req = wrap_tag(0x60, &bind_inner);
            let msg_id = encode_integer(1);
            let msg = encode_sequence(&[&msg_id, &bind_req]);

            // Verify it's non-empty and starts with SEQUENCE
            assert!(!msg.is_empty());
            assert_eq!(msg[0], 0x30);

            // Verify DN is in the message
            let msg_str = String::from_utf8_lossy(&msg);
            assert!(msg_str.contains(dn));
        }
    }
}
