use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use voltrus::license::{LicenseStatus, LicenseTier};

use super::GRACE_DAYS;
use super::activation::LicenseActivation;
use super::verification::evaluate_activation;

fn activation(tier: &str, expires_at: Option<&str>) -> LicenseActivation {
    LicenseActivation {
        version: 1,
        license_key: "VOLTRUS-TEST".to_string(),
        fingerprint_hash: "sha256:x".to_string(),
        issued_at: "2026-01-01T00:00:00Z".to_string(),
        expires_at: expires_at.map(str::to_string),
        tier: Some(tier.to_string()),
        device_limit: None,
        signature: String::new(),
    }
}

fn at(rfc3339: &str) -> OffsetDateTime {
    OffsetDateTime::parse(rfc3339, &Rfc3339).unwrap()
}

#[test]
fn lifetime_key_never_expires() {
    let status = evaluate_activation(&activation("enterprise", None), at("2040-01-01T00:00:00Z"));
    match &status {
        LicenseStatus::Valid {
            expires_at,
            days_remaining,
            grace,
            ..
        } => {
            assert!(expires_at.is_none());
            assert!(days_remaining.is_none());
            assert!(!grace);
        }
        other => panic!("expected Valid, got {other:?}"),
    }
    assert_eq!(status.tier(), LicenseTier::Enterprise);
}

#[test]
fn monthly_key_is_valid_before_expiry() {
    let a = activation("integrator", Some("2026-11-01T00:00:00Z"));
    let status = evaluate_activation(&a, at("2026-10-25T00:00:00Z"));
    match &status {
        LicenseStatus::Valid {
            days_remaining,
            grace,
            ..
        } => {
            assert_eq!(*days_remaining, Some(7));
            assert!(!grace);
        }
        other => panic!("expected Valid, got {other:?}"),
    }
    assert_eq!(status.tier(), LicenseTier::Integrator);
}

#[test]
fn lapsed_monthly_key_keeps_features_for_seven_days() {
    assert_eq!(GRACE_DAYS, 7);
    let a = activation("enterprise", Some("2026-11-01T00:00:00Z"));
    let status = evaluate_activation(&a, at("2026-11-05T00:00:00Z"));
    match &status {
        LicenseStatus::Valid {
            grace,
            days_remaining,
            ..
        } => {
            assert!(grace);
            assert_eq!(*days_remaining, Some(0));
        }
        other => panic!("expected Valid in grace, got {other:?}"),
    }
    assert_eq!(status.tier(), LicenseTier::Enterprise);
}

#[test]
fn monthly_key_falls_back_to_community_after_grace() {
    let a = activation("enterprise", Some("2026-11-01T00:00:00Z"));
    let status = evaluate_activation(&a, at("2026-11-09T00:00:00Z"));
    assert!(matches!(status, LicenseStatus::Expired { .. }));
    assert_eq!(status.tier(), LicenseTier::Community);
    assert_eq!(status.feature_flags()["sso"], false);
}

#[test]
fn legacy_professional_key_becomes_integrator() {
    let status = evaluate_activation(
        &activation("professional", None),
        at("2026-11-01T00:00:00Z"),
    );
    assert_eq!(status.tier(), LicenseTier::Integrator);
}

#[test]
fn legacy_starter_and_edge_keys_become_community_but_stay_valid() {
    for legacy in ["starter", "edge", "engineering"] {
        let status = evaluate_activation(&activation(legacy, None), at("2026-11-01T00:00:00Z"));
        assert!(matches!(status, LicenseStatus::Valid { .. }), "{legacy}");
        assert_eq!(status.tier(), LicenseTier::Community, "{legacy}");
    }
}

#[test]
fn garbage_expiry_is_rejected() {
    let status = evaluate_activation(
        &activation("enterprise", Some("soon")),
        at("2026-11-01T00:00:00Z"),
    );
    assert!(matches!(status, LicenseStatus::InvalidSignature { .. }));
}
