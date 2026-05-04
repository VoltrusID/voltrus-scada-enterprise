use super::*;
use super::tier::LicenseTier;
use super::status::LicenseStatus;

#[test]
fn test_license_tier_max_devices() {
    assert_eq!(LicenseTier::Trial.max_devices(), Some(5));
    assert_eq!(LicenseTier::Starter.max_devices(), Some(50));
    assert_eq!(LicenseTier::Professional.max_devices(), None);
    assert_eq!(LicenseTier::Enterprise.max_devices(), None);
}

#[test]
fn test_license_tier_feature_flags() {
    // Trial tier: limited features
    let trial = LicenseTier::Trial;
    assert_eq!(trial.max_devices(), Some(5));
    assert_eq!(trial.data_retention_days(), 7);
    assert!(!trial.opcua_enabled());
    assert!(!trial.email_notifications());
    assert!(!trial.webhook_notifications());
    assert!(!trial.push_notifications());
    assert!(!trial.sms_notifications());
    assert!(!trial.rest_api_export());
    assert!(trial.csv_export()); // always true
    assert!(!trial.analytics());
    assert!(!trial.oee());
    assert!(!trial.pdf_reporting());
    assert!(!trial.backup_restore());
    assert!(!trial.white_label());
    assert!(!trial.rbac());
    assert!(!trial.multi_site());
    assert!(!trial.recipes());
    assert!(!trial.sso());
    assert!(!trial.sso_saml());
    assert!(!trial.cfr_part_11());
    assert!(!trial.ethernet_ip());
    assert!(!trial.s7_protocol());

    // Starter tier: many features enabled
    let starter = LicenseTier::Starter;
    assert_eq!(starter.max_devices(), Some(50));
    assert_eq!(starter.data_retention_days(), 365);
    assert!(!starter.opcua_enabled());
    assert!(starter.email_notifications());
    assert!(starter.webhook_notifications());
    assert!(starter.push_notifications());
    assert!(!starter.sms_notifications());
    assert!(starter.rest_api_export());
    assert!(starter.csv_export());
    assert!(starter.analytics());
    assert!(starter.oee());
    assert!(!starter.pdf_reporting());
    assert!(starter.backup_restore());
    assert!(starter.white_label());
    assert!(!starter.rbac());
    assert!(!starter.multi_site());
    assert!(!starter.recipes());
    assert!(!starter.sso());
    assert!(!starter.sso_saml());
    assert!(!starter.cfr_part_11());

    // Professional tier: most features enabled
    let pro = LicenseTier::Professional;
    assert_eq!(pro.max_devices(), None);
    assert_eq!(pro.data_retention_days(), 365);
    assert!(pro.opcua_enabled());
    assert!(pro.email_notifications());
    assert!(pro.webhook_notifications());
    assert!(pro.push_notifications());
    assert!(pro.sms_notifications());
    assert!(pro.rest_api_export());
    assert!(pro.csv_export());
    assert!(pro.analytics());
    assert!(pro.oee());
    assert!(pro.pdf_reporting());
    assert!(pro.backup_restore());
    assert!(pro.white_label());
    assert!(pro.rbac());
    assert!(pro.multi_site());
    assert!(pro.recipes());
    assert!(!pro.sso());
    assert!(!pro.sso_saml());
    assert!(!pro.cfr_part_11());

    // Enterprise tier: all features enabled
    let ent = LicenseTier::Enterprise;
    assert_eq!(ent.max_devices(), None);
    assert_eq!(ent.data_retention_days(), 365);
    assert!(ent.opcua_enabled());
    assert!(ent.email_notifications());
    assert!(ent.webhook_notifications());
    assert!(ent.push_notifications());
    assert!(ent.sms_notifications());
    assert!(ent.rest_api_export());
    assert!(ent.csv_export());
    assert!(ent.analytics());
    assert!(ent.oee());
    assert!(ent.pdf_reporting());
    assert!(ent.backup_restore());
    assert!(ent.white_label());
    assert!(ent.rbac());
    assert!(ent.multi_site());
    assert!(ent.recipes());
    assert!(ent.sso());
    assert!(ent.sso_saml());
    assert!(ent.cfr_part_11());
    assert!(ent.ethernet_ip());
    assert!(ent.s7_protocol());
    assert!(ent.rtsp_video());
}

#[test]
fn test_trial_status_is_valid() {
    let tier = LicenseTier::Trial;
    let status = LicenseStatus::Trial {
        started_at: 1000,
        expires_at: 1000 + 30 * 86400,
        days_remaining: 30,
        tier: tier.clone(),
        feature_flags: tier.to_feature_flags(),
    };
    assert!(status.is_valid());
}

#[test]
fn test_trial_expired_is_not_valid() {
    let status = LicenseStatus::TrialExpired {
        started_at: 1000,
        message: "Trial expired".to_string(),
    };
    assert!(!status.is_valid());
}

#[test]
fn test_from_tier_str() {
    assert_eq!(LicenseTier::from_tier_str(None), LicenseTier::Trial);
    assert_eq!(LicenseTier::from_tier_str(Some("")), LicenseTier::Trial);
    assert_eq!(LicenseTier::from_tier_str(Some("unknown")), LicenseTier::Trial);
    assert_eq!(LicenseTier::from_tier_str(Some("starter")), LicenseTier::Starter);
    assert_eq!(LicenseTier::from_tier_str(Some("Starter")), LicenseTier::Starter);
    assert_eq!(LicenseTier::from_tier_str(Some("STARTER")), LicenseTier::Starter);
    assert_eq!(LicenseTier::from_tier_str(Some("professional")), LicenseTier::Professional);
    assert_eq!(LicenseTier::from_tier_str(Some("pro")), LicenseTier::Professional);
    assert_eq!(LicenseTier::from_tier_str(Some("Professional")), LicenseTier::Professional);
    assert_eq!(LicenseTier::from_tier_str(Some("enterprise")), LicenseTier::Enterprise);
    assert_eq!(LicenseTier::from_tier_str(Some("Enterprise")), LicenseTier::Enterprise);
}

#[test]
fn test_to_feature_flags_json_structure() {
    let flags = LicenseTier::Trial.to_feature_flags();
    assert_eq!(flags["max_devices"], 5);
    assert_eq!(flags["data_retention_days"], 7);
    assert_eq!(flags["csv_export"], true);
    assert_eq!(flags["opcua_enabled"], false);

    let flags = LicenseTier::Starter.to_feature_flags();
    assert_eq!(flags["max_devices"], 50);
    assert_eq!(flags["email_notifications"], true);

    let flags = LicenseTier::Enterprise.to_feature_flags();
    assert_eq!(flags["max_devices"], serde_json::Value::Null);
    assert_eq!(flags["sso"], true);
    assert_eq!(flags["cfr_part_11"], true);
    assert_eq!(flags["ethernet_ip"], true);
}

#[test]
fn test_ethernet_ip_enterprise_only() {
    assert!(!LicenseTier::Trial.ethernet_ip());
    assert!(!LicenseTier::Starter.ethernet_ip());
    assert!(!LicenseTier::Professional.ethernet_ip());
    assert!(LicenseTier::Enterprise.ethernet_ip());
}

#[test]
fn test_s7_protocol_enterprise_only() {
    assert!(!LicenseTier::Trial.s7_protocol());
    assert!(!LicenseTier::Starter.s7_protocol());
    assert!(!LicenseTier::Professional.s7_protocol());
    assert!(LicenseTier::Enterprise.s7_protocol());
}

#[test]
fn test_rtsp_video_enterprise_only() {
    assert!(!LicenseTier::Trial.rtsp_video());
    assert!(!LicenseTier::Starter.rtsp_video());
    assert!(!LicenseTier::Professional.rtsp_video());
    assert!(LicenseTier::Enterprise.rtsp_video());
}

#[test]
fn test_theme_full_professional_and_above() {
    assert!(!LicenseTier::Trial.theme_full());
    assert!(!LicenseTier::Starter.theme_full());
    assert!(LicenseTier::Professional.theme_full());
    assert!(LicenseTier::Enterprise.theme_full());
}

#[test]
fn test_per_user_layouts_professional_and_above() {
    assert!(!LicenseTier::Trial.per_user_layouts());
    assert!(!LicenseTier::Starter.per_user_layouts());
    assert!(LicenseTier::Professional.per_user_layouts());
    assert!(LicenseTier::Enterprise.per_user_layouts());
}

#[test]
fn test_white_label_starter_and_above() {
    assert!(!LicenseTier::Trial.white_label());
    assert!(LicenseTier::Starter.white_label());
    assert!(LicenseTier::Professional.white_label());
    assert!(LicenseTier::Enterprise.white_label());
}

#[test]
fn test_trial_status_tier_returns_trial() {
    let tier = LicenseTier::Trial;
    let status = LicenseStatus::Trial {
        started_at: 1000,
        expires_at: 1000 + 30 * 86400,
        days_remaining: 30,
        tier: tier.clone(),
        feature_flags: tier.to_feature_flags(),
    };
    assert_eq!(status.tier(), LicenseTier::Trial);
}

#[test]
fn test_trial_expired_status_tier_returns_trial() {
    let status = LicenseStatus::TrialExpired {
        started_at: 1000,
        message: "Trial expired".to_string(),
    };
    assert_eq!(status.tier(), LicenseTier::Trial);
}

#[test]
fn test_valid_status_tier_maps_correctly() {
    let valid = LicenseStatus::Valid {
        license_key: "key".to_string(),
        expires_at: None,
        days_remaining: None,
        tier: Some("starter".to_string()),
        device_limit: None,
    };
    assert_eq!(valid.tier(), LicenseTier::Starter);
}

#[test]
fn test_license_status_tier_and_feature_flags() {
    // Valid status with tier
    let valid = LicenseStatus::Valid {
        license_key: "key".to_string(),
        expires_at: None,
        days_remaining: None,
        tier: Some("professional".to_string()),
        device_limit: None,
    };
    assert_eq!(valid.tier(), LicenseTier::Professional);
    assert_eq!(valid.feature_flags()["opcua_enabled"], true);

    // Valid status without tier defaults to Trial
    let valid_no_tier = LicenseStatus::Valid {
        license_key: "key".to_string(),
        expires_at: None,
        days_remaining: None,
        tier: None,
        device_limit: None,
    };
    assert_eq!(valid_no_tier.tier(), LicenseTier::Trial);

    // Trial status
    let trial = LicenseStatus::Trial {
        started_at: 1000,
        expires_at: 1000 + 30 * 86400,
        days_remaining: 30,
        tier: LicenseTier::Trial,
        feature_flags: LicenseTier::Trial.to_feature_flags(),
    };
    assert_eq!(trial.tier(), LicenseTier::Trial);
    assert_eq!(trial.feature_flags()["csv_export"], true);

    // Other statuses default to Trial
    let unactivated = LicenseStatus::Unactivated {
        message: "msg".to_string(),
    };
    assert_eq!(unactivated.tier(), LicenseTier::Trial);
    assert_eq!(unactivated.feature_flags(), serde_json::json!({}));
}

#[test]
fn test_sso_saml_enterprise_only() {
    assert!(!LicenseTier::Trial.sso_saml());
    assert!(!LicenseTier::Starter.sso_saml());
    assert!(!LicenseTier::Professional.sso_saml());
    assert!(LicenseTier::Enterprise.sso_saml());
}

#[test]
fn test_sso_saml_in_feature_flags() {
    let flags = LicenseTier::Enterprise.to_feature_flags();
    assert_eq!(flags["sso_saml"], true);

    let flags = LicenseTier::Trial.to_feature_flags();
    assert_eq!(flags["sso_saml"], false);

    let flags = LicenseTier::Professional.to_feature_flags();
    assert_eq!(flags["sso_saml"], false);
}
