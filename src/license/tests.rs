use super::status::LicenseStatus;
use super::tier::LicenseTier;
use super::*;

#[test]
fn test_license_tier_max_devices() {
    assert_eq!(LicenseTier::Engineering.max_devices(), Some(5));
    assert_eq!(LicenseTier::Edge.max_devices(), Some(15));
    assert_eq!(LicenseTier::Starter.max_devices(), Some(50));
    assert_eq!(LicenseTier::Professional.max_devices(), None);
    assert_eq!(LicenseTier::Enterprise.max_devices(), None);
}

#[test]
fn test_license_tier_feature_flags() {
    // Engineering tier: limited features
    let engineering = LicenseTier::Engineering;
    assert_eq!(engineering.max_devices(), Some(5));
    assert_eq!(engineering.data_retention_days(), 7);
    assert!(!engineering.opcua_enabled());
    assert!(!engineering.email_notifications());
    assert!(!engineering.webhook_notifications());
    assert!(!engineering.push_notifications());
    assert!(!engineering.sms_notifications());
    assert!(!engineering.rest_api_export());
    assert!(engineering.csv_export()); // always true
    assert!(!engineering.analytics());
    assert!(!engineering.oee());
    assert!(!engineering.pdf_reporting());
    assert!(!engineering.backup_restore());
    assert!(!engineering.white_label());
    assert!(!engineering.rbac());
    assert!(!engineering.multi_site());
    assert!(!engineering.recipes());
    assert!(!engineering.sso());
    assert!(!engineering.sso_saml());
    assert!(!engineering.cfr_part_11());
    assert!(!engineering.ethernet_ip());
    assert!(!engineering.s7_protocol());

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

    // Edge tier: same features as Starter but capped devices and retention
    let edge = LicenseTier::Edge;
    assert_eq!(edge.max_devices(), Some(15));
    assert_eq!(edge.data_retention_days(), 90);
    assert!(!edge.opcua_enabled());
    assert!(!edge.sms_notifications());
    assert!(!edge.pdf_reporting());
    assert!(!edge.rbac());
    assert!(!edge.multi_site());
    assert!(!edge.recipes());
    assert!(!edge.sso());
    assert!(!edge.sso_saml());
    assert!(!edge.cfr_part_11());
    assert!(edge.email_notifications());
    assert!(edge.rest_api_export());
    assert!(edge.csv_export());
    assert!(edge.analytics());
    assert!(edge.oee());
    assert!(edge.backup_restore());
    assert!(edge.white_label());

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
fn test_engineering_status_is_valid() {
    let tier = LicenseTier::Engineering;
    let status = LicenseStatus::Engineering {
        started_at: 1000,
        polling_elapsed_secs: 0,
        polling_remaining_secs: 14400,
        daily_budget_secs: 14400,
        daily_used_secs: 0,
        polling_active: false,
        tier: tier.clone(),
        feature_flags: tier.to_feature_flags(),
    };
    assert!(status.is_valid());
}

#[test]
fn test_engineering_polling_stopped_is_valid() {
    let status = LicenseStatus::EngineeringPollingStopped {
        started_at: 1000,
        daily_budget_secs: 14400,
        daily_used_secs: 14400,
        message: "Polling stopped".to_string(),
    };
    assert!(status.is_valid());
}

#[test]
fn test_from_tier_str() {
    assert_eq!(LicenseTier::from_tier_str(None), LicenseTier::Engineering);
    assert_eq!(
        LicenseTier::from_tier_str(Some("")),
        LicenseTier::Engineering
    );
    assert_eq!(
        LicenseTier::from_tier_str(Some("unknown")),
        LicenseTier::Engineering
    );
    assert_eq!(
        LicenseTier::from_tier_str(Some("starter")),
        LicenseTier::Starter
    );
    assert_eq!(
        LicenseTier::from_tier_str(Some("Starter")),
        LicenseTier::Starter
    );
    assert_eq!(LicenseTier::from_tier_str(Some("edge")), LicenseTier::Edge);
    assert_eq!(LicenseTier::from_tier_str(Some("Edge")), LicenseTier::Edge);
    assert_eq!(
        LicenseTier::from_tier_str(Some("STARTER")),
        LicenseTier::Starter
    );
    assert_eq!(
        LicenseTier::from_tier_str(Some("professional")),
        LicenseTier::Professional
    );
    assert_eq!(
        LicenseTier::from_tier_str(Some("pro")),
        LicenseTier::Professional
    );
    assert_eq!(
        LicenseTier::from_tier_str(Some("Professional")),
        LicenseTier::Professional
    );
    assert_eq!(
        LicenseTier::from_tier_str(Some("enterprise")),
        LicenseTier::Enterprise
    );
    assert_eq!(
        LicenseTier::from_tier_str(Some("Enterprise")),
        LicenseTier::Enterprise
    );
}

#[test]
fn test_to_feature_flags_json_structure() {
    let flags = LicenseTier::Engineering.to_feature_flags();
    assert_eq!(flags["max_devices"], 5);
    assert_eq!(flags["data_retention_days"], 7);
    assert_eq!(flags["csv_export"], true);
    assert_eq!(flags["opcua_enabled"], false);

    let flags = LicenseTier::Starter.to_feature_flags();
    assert_eq!(flags["max_devices"], 50);
    assert_eq!(flags["email_notifications"], true);

    let flags = LicenseTier::Edge.to_feature_flags();
    assert_eq!(flags["max_devices"], 15);
    assert_eq!(flags["data_retention_days"], 90);
    assert_eq!(flags["email_notifications"], true);
    assert_eq!(flags["opcua_enabled"], false);

    let flags = LicenseTier::Enterprise.to_feature_flags();
    assert_eq!(flags["max_devices"], serde_json::Value::Null);
    assert_eq!(flags["sso"], true);
    assert_eq!(flags["cfr_part_11"], true);
    assert_eq!(flags["ethernet_ip"], true);
}

#[test]
fn test_ethernet_ip_enterprise_only() {
    assert!(!LicenseTier::Engineering.ethernet_ip());
    assert!(!LicenseTier::Starter.ethernet_ip());
    assert!(!LicenseTier::Professional.ethernet_ip());
    assert!(LicenseTier::Enterprise.ethernet_ip());
}

#[test]
fn test_s7_protocol_enterprise_only() {
    assert!(!LicenseTier::Engineering.s7_protocol());
    assert!(!LicenseTier::Starter.s7_protocol());
    assert!(!LicenseTier::Professional.s7_protocol());
    assert!(LicenseTier::Enterprise.s7_protocol());
}

#[test]
fn test_rtsp_video_enterprise_only() {
    assert!(!LicenseTier::Engineering.rtsp_video());
    assert!(!LicenseTier::Starter.rtsp_video());
    assert!(!LicenseTier::Professional.rtsp_video());
    assert!(LicenseTier::Enterprise.rtsp_video());
}

#[test]
fn test_theme_full_professional_and_above() {
    assert!(!LicenseTier::Engineering.theme_full());
    assert!(!LicenseTier::Starter.theme_full());
    assert!(LicenseTier::Professional.theme_full());
    assert!(LicenseTier::Enterprise.theme_full());
}

#[test]
fn test_per_user_layouts_professional_and_above() {
    assert!(!LicenseTier::Engineering.per_user_layouts());
    assert!(!LicenseTier::Starter.per_user_layouts());
    assert!(LicenseTier::Professional.per_user_layouts());
    assert!(LicenseTier::Enterprise.per_user_layouts());
}

#[test]
fn test_white_label_starter_and_above() {
    assert!(!LicenseTier::Engineering.white_label());
    assert!(LicenseTier::Starter.white_label());
    assert!(LicenseTier::Professional.white_label());
    assert!(LicenseTier::Enterprise.white_label());
}

#[test]
fn test_engineering_status_tier_returns_engineering() {
    let tier = LicenseTier::Engineering;
    let status = LicenseStatus::Engineering {
        started_at: 1000,
        polling_elapsed_secs: 0,
        polling_remaining_secs: 14400,
        daily_budget_secs: 14400,
        daily_used_secs: 0,
        polling_active: false,
        tier: tier.clone(),
        feature_flags: tier.to_feature_flags(),
    };
    assert_eq!(status.tier(), LicenseTier::Engineering);
}

#[test]
fn test_engineering_polling_stopped_status_tier_returns_engineering() {
    let status = LicenseStatus::EngineeringPollingStopped {
        started_at: 1000,
        daily_budget_secs: 14400,
        daily_used_secs: 14400,
        message: "Polling stopped".to_string(),
    };
    assert_eq!(status.tier(), LicenseTier::Engineering);
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

    // Valid status without tier defaults to Engineering
    let valid_no_tier = LicenseStatus::Valid {
        license_key: "key".to_string(),
        expires_at: None,
        days_remaining: None,
        tier: None,
        device_limit: None,
    };
    assert_eq!(valid_no_tier.tier(), LicenseTier::Engineering);

    // Engineering status
    let engineering = LicenseStatus::Engineering {
        started_at: 1000,
        polling_elapsed_secs: 0,
        polling_remaining_secs: 14400,
        daily_budget_secs: 14400,
        daily_used_secs: 0,
        polling_active: false,
        tier: LicenseTier::Engineering,
        feature_flags: LicenseTier::Engineering.to_feature_flags(),
    };
    assert_eq!(engineering.tier(), LicenseTier::Engineering);
    assert_eq!(engineering.feature_flags()["csv_export"], true);

    // Other statuses default to Engineering
    let unactivated = LicenseStatus::Unactivated {
        message: "msg".to_string(),
    };
    assert_eq!(unactivated.tier(), LicenseTier::Engineering);
    assert_eq!(unactivated.feature_flags(), serde_json::json!({}));
}

#[test]
fn test_sso_saml_enterprise_only() {
    assert!(!LicenseTier::Engineering.sso_saml());
    assert!(!LicenseTier::Starter.sso_saml());
    assert!(!LicenseTier::Professional.sso_saml());
    assert!(LicenseTier::Enterprise.sso_saml());
}

#[test]
fn test_sso_saml_in_feature_flags() {
    let flags = LicenseTier::Enterprise.to_feature_flags();
    assert_eq!(flags["sso_saml"], true);

    let flags = LicenseTier::Engineering.to_feature_flags();
    assert_eq!(flags["sso_saml"], false);

    let flags = LicenseTier::Professional.to_feature_flags();
    assert_eq!(flags["sso_saml"], false);
}

#[test]
fn test_hmac_sign_and_verify() {
    let date = "2026-05-17";
    let used_secs: i64 = 3600;

    // Sign and verify should succeed
    let sig = super::verification::sign_polling_state(date, used_secs);
    assert!(super::verification::verify_polling_hmac(
        date, used_secs, &sig
    ));

    // Wrong used_secs should fail
    assert!(!super::verification::verify_polling_hmac(date, 3601, &sig));

    // Wrong date should fail
    assert!(!super::verification::verify_polling_hmac(
        "2026-05-18",
        used_secs,
        &sig
    ));

    // Garbage HMAC should fail
    assert!(!super::verification::verify_polling_hmac(
        date, used_secs, "deadbeef"
    ));
}

#[test]
fn test_daily_budget_constants() {
    assert_eq!(super::verification::ENGINEERING_DAILY_BUDGET_SECS, 14400); // 4 hours
    assert_eq!(super::verification::ENGINEERING_POLLING_TICK_SECS, 60);
}
