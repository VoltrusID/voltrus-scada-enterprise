use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use voltrus::license::{LicenseStatus, LicenseTier, SupportState};

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
        site_id: None,
        site_name: None,
        support_until: None,
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
fn legacy_starter_and_edge_keep_logo_only_white_label() {
    for legacy in ["starter", "edge", " Starter "] {
        let status = evaluate_activation(&activation(legacy, None), at("2026-11-01T00:00:00Z"));
        assert!(status.white_label(), "{legacy}");
        let flags = status.feature_flags();
        for (key, value) in flags.as_object().unwrap() {
            assert_eq!(
                value,
                &serde_json::json!(key == "white_label"),
                "{legacy} {key}"
            );
        }
    }
}

#[test]
fn other_tiers_get_no_extra_white_label_grant() {
    let now = at("2026-11-01T00:00:00Z");
    let engineering = evaluate_activation(&activation("engineering", None), now);
    assert!(!engineering.white_label());
    let integrator = evaluate_activation(&activation("integrator", None), now);
    assert!(integrator.white_label());
    assert_eq!(integrator.feature_flags()["theme_full"], true);
}

#[test]
fn expired_legacy_starter_loses_the_logo_grant() {
    let a = activation("starter", Some("2026-11-01T00:00:00Z"));
    let status = evaluate_activation(&a, at("2026-11-09T00:00:00Z"));
    assert!(matches!(status, LicenseStatus::Expired { .. }));
    assert!(!status.white_label());
}

#[test]
fn garbage_expiry_is_rejected() {
    let status = evaluate_activation(
        &activation("enterprise", Some("soon")),
        at("2026-11-01T00:00:00Z"),
    );
    assert!(matches!(status, LicenseStatus::InvalidSignature { .. }));
}

// ── Per-site licence and support contract ──

/// Unix seconds of 2027-01-01T00:00:00Z.
const SUPPORT_END: i64 = 1_798_761_600;

fn per_site(support_until: Option<i64>) -> LicenseActivation {
    LicenseActivation {
        site_id: Some("0b0e8a5c-1f3a-4f6e-9d0a-5d1d8f4c7a11".to_string()),
        site_name: Some("Plant 1".to_string()),
        support_until,
        ..activation("integrator", None)
    }
}

fn valid_fields(status: &LicenseStatus) -> (Option<&voltrus::license::LicenseSite>, Option<i64>) {
    match status {
        LicenseStatus::Valid {
            site,
            support_until,
            ..
        } => (site.as_ref(), *support_until),
        other => panic!("expected Valid, got {other:?}"),
    }
}

#[test]
fn per_site_key_reports_its_site_and_support_end() {
    let status = evaluate_activation(&per_site(Some(SUPPORT_END)), at("2026-10-28T00:00:00Z"));
    let (site, support_until) = valid_fields(&status);
    let site = site.expect("site");
    assert_eq!(site.id, "0b0e8a5c-1f3a-4f6e-9d0a-5d1d8f4c7a11");
    assert_eq!(site.name, "Plant 1");
    assert_eq!(support_until, Some(SUPPORT_END));
    assert_eq!(
        status.support_at(at("2026-10-28T00:00:00Z").unix_timestamp()),
        SupportState::Active
    );
    assert_eq!(status.tier(), LicenseTier::Integrator);
}

#[test]
fn key_issued_before_per_site_licensing_reports_neither() {
    let status = evaluate_activation(&activation("enterprise", None), at("2030-01-01T00:00:00Z"));
    assert_eq!(valid_fields(&status), (None, None));
    assert_eq!(
        status.support_at(at("2030-01-01T00:00:00Z").unix_timestamp()),
        SupportState::None
    );
}

#[test]
fn lapsed_support_never_limits_the_licence() {
    // Years after support ended, on a perpetual per-site key: still Valid, same tier,
    // same features as while support was active. Only the reported state changes.
    let now = at("2031-06-01T00:00:00Z");
    let lapsed = evaluate_activation(&per_site(Some(SUPPORT_END)), now);
    let active = evaluate_activation(&per_site(Some(SUPPORT_END)), at("2026-10-28T00:00:00Z"));
    assert!(matches!(lapsed, LicenseStatus::Valid { grace: false, .. }));
    assert_eq!(
        lapsed.support_at(now.unix_timestamp()),
        SupportState::Lapsed
    );
    assert_eq!(lapsed.tier(), LicenseTier::Integrator);
    assert_eq!(lapsed.tier(), active.tier());
    assert_eq!(lapsed.feature_flags(), active.feature_flags());
    assert_eq!(valid_fields(&lapsed).1, Some(SUPPORT_END));
}

#[test]
fn support_ends_on_the_second_it_says() {
    let status = evaluate_activation(&per_site(Some(SUPPORT_END)), at("2026-10-28T00:00:00Z"));
    assert_eq!(status.support_at(SUPPORT_END - 1), SupportState::Active);
    assert_eq!(status.support_at(SUPPORT_END), SupportState::Lapsed);
}

#[test]
fn site_id_without_a_name_shows_the_id() {
    let mut a = per_site(None);
    a.site_name = None;
    let status = evaluate_activation(&a, at("2026-10-28T00:00:00Z"));
    let (site, _) = valid_fields(&status);
    assert_eq!(site.unwrap().name, "0b0e8a5c-1f3a-4f6e-9d0a-5d1d8f4c7a11");
}

#[test]
fn a_name_without_an_id_is_not_a_site() {
    let mut a = per_site(None);
    a.site_id = None;
    let status = evaluate_activation(&a, at("2026-10-28T00:00:00Z"));
    assert_eq!(valid_fields(&status).0, None);
}

#[test]
fn support_does_not_change_the_monthly_expiry_rules() {
    // A monthly key past its grace period is Expired whatever the support contract says.
    let mut a = per_site(Some(SUPPORT_END));
    a.expires_at = Some("2026-11-01T00:00:00Z".to_string());
    let status = evaluate_activation(&a, at("2026-11-09T00:00:00Z"));
    assert!(matches!(status, LicenseStatus::Expired { .. }));
}
