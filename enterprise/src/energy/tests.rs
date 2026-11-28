//! End to end on an in-memory core database: historian samples → hourly job →
//! `energy_hour` → bill, alarms and the daily summary.

use time::{Date, Month};
use voltrus::storage::Storage;

use super::counter::Quality;
use super::job;
use super::report;
use super::store::{self, Meter, MeterRole, SiteFields};
use super::tariff::{AlertConfig, Band, Golongan, LocalClock, Site, TariffValues, WbpWindow};

const DEV: &str = "pm5560";

async fn setup(alerts: AlertConfig) -> (Storage, Site, Meter) {
    let storage = Storage::new_in_memory().unwrap();
    storage
        .with_conn(|c| Ok(super::ensure_schema(c)?))
        .await
        .unwrap();
    let (site, meter) = storage
        .with_conn(move |c| {
            let id = store::insert_site(
                c,
                &SiteFields {
                    name: "Pabrik A",
                    timezone: "Asia/Jakarta",
                    wbp_windows: &WbpWindow::default_windows(),
                    contracted_kva: Some(2000.0),
                    alerts: &alerts,
                },
                0,
            )?;
            store::upsert_tariff(
                c,
                id,
                "2026-01",
                &TariffValues::defaults(Golongan::I3),
                "",
                0,
            )?;
            let mut m = Meter {
                id: 0,
                site_id: id,
                name: "Incomer".into(),
                device_id: DEV.into(),
                kwh_metric: "Energy_Import".into(),
                kvarh_metric: Some("Energy_Reactive".into()),
                kvar_metric: None,
                role: MeterRole::PlnIncomer,
                // CT 400/5: the counter counts 1/80 of the real energy.
                multiplier: 80.0,
                rollover: None,
                max_gap_secs: 900,
                enabled: true,
                created_at: 0,
                updated_at: 0,
            };
            m.id = store::insert_meter(c, &m, 0)?;
            Ok((store::get_site(c, id)?.unwrap(), m))
        })
        .await
        .unwrap();
    (storage, site, meter)
}

fn midnight_wib() -> i64 {
    LocalClock::parse("Asia/Jakarta")
        .unwrap()
        .midnight(Date::from_calendar_date(2026, Month::November, 2).unwrap())
}

/// One sample a minute: 100 kWh and 75 kVArh an hour after the ×80 multiplier
/// (cos φ 0.80), minutes `first..=last` after `start`.
async fn write_minutes(storage: &Storage, start: i64, first: i64, last: i64) {
    let mut rows = Vec::new();
    for i in first..=last {
        let t = start + i * 60;
        rows.push((
            DEV.to_string(),
            t,
            "Energy_Import".to_string(),
            5000.0 + 1.25 * i as f64 / 60.0,
        ));
        rows.push((
            DEV.to_string(),
            t,
            "Energy_Reactive".to_string(),
            300.0 + 0.9375 * i as f64 / 60.0,
        ));
    }
    storage.insert_batch(&rows).await.unwrap();
}

async fn write_samples(storage: &Storage, start: i64, hours: i64) {
    write_minutes(storage, start, 0, hours * 60).await;
}

#[tokio::test]
async fn counters_become_hours_with_multiplier_and_wbp_band() {
    let (storage, site, meter) = setup(AlertConfig::default()).await;
    let day = midnight_wib();
    write_samples(&storage, day, 26).await;
    let n = job::compute_meter(&storage, &site, &meter, day, day + 86_400, day + 26 * 3600)
        .await
        .unwrap();
    assert_eq!(n, 24);
    let rows = storage
        .with_conn(move |c| store::hours_for_meters(c, &[meter.id], day, day + 86_400))
        .await
        .unwrap();
    assert!(
        rows.iter()
            .all(|r| (r.kwh - 100.0).abs() < 1e-6 && r.quality == Quality::Ok)
    );
    assert!(rows.iter().all(|r| (r.kvarh.unwrap() - 75.0).abs() < 1e-6));
    let wbp: Vec<_> = rows.iter().filter(|r| r.band == Band::Wbp).collect();
    assert_eq!(wbp.len(), 4);
    assert_eq!(wbp[0].hour_start, day + 18 * 3600);

    // Idempotent: a second run gives the same rows.
    job::compute_meter(&storage, &site, &meter, day, day + 86_400, day + 26 * 3600)
        .await
        .unwrap();
    let again = storage
        .with_conn(move |c| store::hours_for_meters(c, &[meter.id], day, day + 86_400))
        .await
        .unwrap();
    assert_eq!(rows, again);

    let tariffs = storage
        .with_conn(move |c| store::list_tariffs(c, 1))
        .await
        .unwrap();
    let days = report::daily(&site, &tariffs, "2026-11", &rows);
    assert_eq!(days.len(), 1);
    assert!((days[0].kwh_wbp - 400.0).abs() < 1e-6);
    // 400 kWh × (1.4 − 1) × 1035.78
    assert!((days[0].wbp_premium - 165_724.8).abs() < 0.01);
    // 1800 kVArh − 0.6197 × 2400 kWh, × 1114.74
    let excess = 1800.0 - 0.85f64.acos().tan() * 2400.0;
    assert!((days[0].kvarh_charge.unwrap() - excess * 1114.74).abs() < 0.01);
}

#[tokio::test]
async fn a_late_hour_is_a_gap_until_data_arrives() {
    let (storage, site, meter) = setup(AlertConfig::default()).await;
    let day = midnight_wib();
    write_samples(&storage, day, 2).await;
    // At 02:30 the gateway has delivered only up to 02:00: hour 02 has no end.
    job::compute_meter(
        &storage,
        &site,
        &meter,
        day,
        day + 3 * 3600,
        day + 2 * 3600 + 1800,
    )
    .await
    .unwrap();
    let id = meter.id;
    let q = |s: Storage| async move {
        s.with_conn(move |c| store::hours_for_meters(c, &[id], day, day + 3 * 3600))
            .await
            .unwrap()
    };
    let rows = q(storage.clone()).await;
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[2].quality, Quality::Gap);
    // The data arrives; the recompute fixes the hour.
    write_minutes(&storage, day, 121, 5 * 60).await;
    job::compute_meter(&storage, &site, &meter, day, day + 3 * 3600, day + 5 * 3600)
        .await
        .unwrap();
    let rows = q(storage.clone()).await;
    assert!(rows.iter().all(|r| r.quality == Quality::Ok), "{rows:?}");
}

#[tokio::test]
async fn low_power_factor_and_wbp_load_raise_alarms_and_the_summary_is_sent_once() {
    let alerts = AlertConfig {
        wbp_kw_target: Some(50.0),
        ..AlertConfig::default()
    };
    let (storage, site, _meter) = setup(alerts).await;
    let day = midnight_wib();
    write_samples(&storage, day, 25).await;

    // 22:02 WIB: the last full hour (21:00) is WBP at 100 kW, cos φ 0.80.
    job::run_cycle(&storage, None, None, day + 22 * 3600 + 120)
        .await
        .unwrap();
    assert!(
        storage
            .has_any_unresolved_alarm(&job::pf_key(site.id))
            .await
            .unwrap()
    );
    assert!(
        storage
            .has_any_unresolved_alarm(&job::wbp_kw_key(site.id))
            .await
            .unwrap()
    );
    // Raised once, not every run.
    job::run_cycle(&storage, None, None, day + 22 * 3600 + 300)
        .await
        .unwrap();
    let open = storage.list_alarms(None, 50).await.unwrap();
    assert_eq!(open.len(), 2, "{open:?}");

    // 23:02: the last hour is LWBP, so the WBP load alarm clears.
    job::run_cycle(&storage, None, None, day + 23 * 3600 + 120)
        .await
        .unwrap();
    assert!(
        !storage
            .has_any_unresolved_alarm(&job::wbp_kw_key(site.id))
            .await
            .unwrap()
    );
    assert!(
        storage
            .has_any_unresolved_alarm(&job::pf_key(site.id))
            .await
            .unwrap()
    );

    // Daily summary: needs a notification rule for the site's daily key.
    storage
        .create_notification_setting(&job::daily_key(site.id), "telegram", "{}", true)
        .await
        .unwrap();
    job::run_cycle(&storage, None, None, day + 86_400 + 120)
        .await
        .unwrap();
    let sid = site.id;
    let sent = storage
        .with_conn(move |c| store::notice_sent(c, sid, "daily", "2026-11-02"))
        .await
        .unwrap();
    assert!(sent);
}

#[test]
fn summary_line_in_both_languages() {
    let mut site = Site {
        id: 1,
        name: "Pabrik A".into(),
        timezone: "Asia/Jakarta".into(),
        wbp_windows: WbpWindow::default_windows(),
        contracted_kva: None,
        alerts: AlertConfig::default(),
        created_at: 0,
        updated_at: 0,
    };
    let day = report::DayRow {
        date: "2026-11-27".into(),
        kwh_lwbp: 16_000.0,
        kwh_wbp: 4_000.0,
        kvarh: Some(15_000.0),
        cos_phi: Some(0.8),
        wbp_premium: 1_657_248.0,
        kvarh_charge: Some(2_898_324.0),
        cost_total: 0.0,
        quality: report::QualityCounts::default(),
    };
    let id = job::summary_line(&site, "2026-11-27", &day);
    assert!(id.contains("biaya kelebihan kVArh Rp 2.898.324"), "{id}");
    assert!(id.contains("cos φ 0,80"));
    site.alerts.lang = "en".into();
    let en = job::summary_line(&site, "2026-11-27", &day);
    assert!(en.contains("kVArh charge Rp 2,898,324"), "{en}");
}
