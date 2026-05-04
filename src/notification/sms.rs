use anyhow::Result;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

use crate::models::SmtpConfig;

/// Maximum SMS message length (GSM 7-bit standard).
const SMS_MAX_LENGTH: usize = 160;

/// Carrier email-to-SMS gateway domains.
///
/// Maps carrier identifiers to the email gateway used for email-to-SMS delivery.
/// These gateways accept standard SMTP email and deliver as SMS/MMS to the
/// recipient's mobile phone.
pub fn carrier_gateway(carrier: &str) -> Option<&'static str> {
    // Normalize: lowercase and strip hyphens/spaces for flexible matching
    let normalized = carrier.to_lowercase().replace(['-', ' '], "");
    match normalized.as_str() {
        "att" => Some("txt.att.net"),
        "verizon" => Some("vtext.com"),
        "tmobile" => Some("tmomail.net"),
        "sprint" => Some("messaging.sprintpcs.com"),
        "uscellular" => Some("email.uscc.net"),
        _ => None,
    }
}

/// Truncate a message to fit within the SMS character limit.
///
/// If the message exceeds `SMS_MAX_LENGTH`, it is truncated and "..." is appended
/// to indicate truncation (still within the limit).
pub fn format_sms_message(message: &str) -> String {
    if message.len() <= SMS_MAX_LENGTH {
        message.to_string()
    } else {
        let truncated = &message[..SMS_MAX_LENGTH - 3];
        format!("{}...", truncated)
    }
}

/// Send an SMS alarm notification via email-to-SMS gateway.
///
/// This constructs an email addressed to `{phone_number}@{carrier_gateway_domain}`
/// with a plain-text body containing the alarm message (truncated to 160 chars).
///
/// # Arguments
/// * `smtp_config` - SMTP server configuration for sending the email
/// * `phone_number` - Recipient phone number (digits only, e.g. "5551234567")
/// * `carrier` - Carrier identifier (e.g. "att", "verizon", "tmobile")
/// * `message` - Alarm message text (will be truncated to 160 chars if needed)
pub async fn send_sms_alarm(
    smtp_config: &SmtpConfig,
    phone_number: &str,
    carrier: &str,
    message: &str,
) -> Result<()> {
    let gateway =
        carrier_gateway(carrier).ok_or_else(|| anyhow::anyhow!("Unknown carrier: {}", carrier))?;

    let to_address = format!("{}@{}", phone_number, gateway);

    let sms_body = format_sms_message(message);

    let mailer = build_smtp_transport(smtp_config)?;

    let from = format!("{} <{}>", smtp_config.from_name, smtp_config.from_address)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid from address: {}", e))?;

    let recipient = to_address
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid SMS gateway address: {}", e))?;

    let email = Message::builder()
        .from(from)
        .to(recipient)
        .subject("Voltrus Alarm")
        .singlepart(
            lettre::message::SinglePart::builder()
                .header(ContentType::TEXT_PLAIN)
                .body(sms_body),
        )
        .map_err(|e| anyhow::anyhow!("Failed to build SMS email message: {}", e))?;

    mailer
        .send(email)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send SMS via gateway: {}", e))?;

    tracing::info!(
        "SMS alarm sent to {}@{} [{}]",
        phone_number,
        carrier,
        gateway,
    );

    Ok(())
}

/// Build an SMTP transport from the given config.
fn build_smtp_transport(config: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let mailer = match config.encryption {
        crate::models::SmtpEncryption::StartTls => {
            let addr = format!("{}:{}", config.host, config.port);
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&addr)
                .map_err(|_e| anyhow::anyhow!("Failed to create STARTTLS transport for {}", addr))?
                .credentials(Credentials::new(
                    config.username.clone(),
                    config.password.clone(),
                ))
                .build()
        }
        crate::models::SmtpEncryption::ImplicitTls => {
            let addr = format!("{}:{}", config.host, config.port);
            AsyncSmtpTransport::<Tokio1Executor>::relay(&addr)
                .map_err(|_e| anyhow::anyhow!("Failed to create TLS transport for {}", addr))?
                .credentials(Credentials::new(
                    config.username.clone(),
                    config.password.clone(),
                ))
                .build()
        }
        crate::models::SmtpEncryption::None => {
            let addr = format!("{}:{}", config.host, config.port);
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&addr)
                .credentials(Credentials::new(
                    config.username.clone(),
                    config.password.clone(),
                ))
                .build()
        }
    };

    Ok(mailer)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carrier_gateway_known() {
        assert_eq!(carrier_gateway("att"), Some("txt.att.net"));
        assert_eq!(carrier_gateway("verizon"), Some("vtext.com"));
        assert_eq!(carrier_gateway("tmobile"), Some("tmomail.net"));
        assert_eq!(carrier_gateway("sprint"), Some("messaging.sprintpcs.com"));
        assert_eq!(carrier_gateway("uscellular"), Some("email.uscc.net"));
    }

    #[test]
    fn test_carrier_gateway_case_insensitive() {
        assert_eq!(carrier_gateway("ATT"), Some("txt.att.net"));
        assert_eq!(carrier_gateway("Verizon"), Some("vtext.com"));
        assert_eq!(carrier_gateway("T-Mobile"), Some("tmomail.net"));
        assert_eq!(carrier_gateway("SPRINT"), Some("messaging.sprintpcs.com"));
        assert_eq!(carrier_gateway("USCellular"), Some("email.uscc.net"));
    }

    #[test]
    fn test_carrier_gateway_unknown() {
        assert_eq!(carrier_gateway("unknown"), None);
        assert_eq!(carrier_gateway(""), None);
        assert_eq!(carrier_gateway("telkomsel"), None);
        assert_eq!(carrier_gateway("orange"), None);
    }

    #[test]
    fn test_format_sms_message_short() {
        let msg = "Voltage high";
        assert_eq!(format_sms_message(msg), "Voltage high");
    }

    #[test]
    fn test_format_sms_message_exact_limit() {
        let msg = "a".repeat(160);
        let result = format_sms_message(&msg);
        assert_eq!(result.len(), 160);
        assert_eq!(result, msg);
    }

    #[test]
    fn test_format_sms_message_truncation() {
        let msg = "a".repeat(200);
        let result = format_sms_message(&msg);
        assert_eq!(result.len(), 160);
        assert!(result.ends_with("..."));
        assert!(result.starts_with("a".repeat(157).as_str()));
    }

    #[test]
    fn test_format_sms_message_one_over_limit() {
        let msg = "a".repeat(161);
        let result = format_sms_message(&msg);
        assert_eq!(result.len(), 160);
        assert!(result.ends_with("..."));
    }
}
