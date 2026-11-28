//! Month-to-date bill, projected month-end and the daily breakdown of a site,
//! from the `energy_hour` rows of its PLN incomer meters.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::bill::{self, Bill, HourCounts, PlnFigures, Quantities, Reconciliation};
use super::counter::Quality;
use super::store::HourRow;
use super::tariff::{Band, Site, Tariff, format_date, tariff_for};

/// What the shadow bill leaves out, as the worksheet does.
pub const EXCLUDED: [&str; 3] = ["demand_charge", "ppj", "ppn"];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityCounts {
    pub ok: u32,
    pub estimated: u32,
    pub gap: u32,
}

impl QualityCounts {
    fn add(&mut self, q: Quality) {
        match q {
            Quality::Ok => self.ok += 1,
            Quality::Estimated => self.estimated += 1,
            Quality::Gap => self.gap += 1,
        }
    }
}

/// Sum of the rows by band. kVArh is `None` when no row carries it.
pub fn quantities(rows: &[HourRow]) -> Quantities {
    let mut q = Quantities::default();
    for r in rows {
        match r.band {
            Band::Lwbp => q.kwh_lwbp += r.kwh,
            Band::Wbp => q.kwh_wbp += r.kwh,
        }
        if let Some(k) = r.kvarh {
            q.kvarh = Some(q.kvarh.unwrap_or(0.0) + k);
        }
    }
    q
}

/// Per hour (several meters share an hour): the worst quality.
fn hour_qualities(rows: &[HourRow]) -> BTreeMap<i64, (Band, Quality)> {
    let mut hours: BTreeMap<i64, (Band, Quality)> = BTreeMap::new();
    for r in rows {
        hours
            .entry(r.hour_start)
            .and_modify(|e| e.1 = e.1.worst(r.quality))
            .or_insert((r.band, r.quality));
    }
    hours
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthReport {
    pub site_id: i64,
    pub site_name: String,
    pub month: String,
    pub timezone: String,
    /// UTC seconds, `[from, to)`.
    pub from: i64,
    pub to: i64,
    /// The month is over (every hour could have been computed).
    pub complete: bool,
    /// Hours of the month by band.
    pub hours_in_month: HourCounts,
    /// Hours with computed data by band.
    pub hours_with_data: HourCounts,
    pub quality: QualityCounts,
    pub incomer_meters: Vec<i64>,
    pub tariff: Option<Tariff>,
    pub month_to_date: Option<Bill>,
    pub projected: Option<Bill>,
    pub pln: Option<PlnFigures>,
    pub reconciliation: Option<Reconciliation>,
    /// A reconciled month agreed with PLN within 2%: the tariff setup and the
    /// excess-kVArh rule are confirmed for this site.
    pub confirmed: bool,
    pub excluded: Vec<String>,
}

/// `rows` are the incomer rows of the month.
pub fn month_report(
    site: &Site,
    tariffs: &[Tariff],
    month: &str,
    rows: &[HourRow],
    incomer_meters: Vec<i64>,
    now: i64,
) -> Result<MonthReport, String> {
    let clock = site.clock();
    let (from, to) = clock.month_range(month)?;
    let mut in_month = HourCounts::default();
    let mut h = from;
    while h < to {
        match clock.band(h, &site.wbp_windows) {
            Band::Lwbp => in_month.lwbp += 1,
            Band::Wbp => in_month.wbp += 1,
        }
        h += 3600;
    }
    let hours = hour_qualities(rows);
    let mut seen = HourCounts::default();
    let mut quality = QualityCounts::default();
    for (band, q) in hours.values() {
        match band {
            Band::Lwbp => seen.lwbp += 1,
            Band::Wbp => seen.wbp += 1,
        }
        quality.add(*q);
    }
    let tariff = tariff_for(tariffs, month).cloned();
    let (mtd, projected) = match (&tariff, rows.is_empty()) {
        (Some(t), false) => {
            let q = quantities(rows);
            let remaining = HourCounts {
                lwbp: in_month.lwbp.saturating_sub(seen.lwbp),
                wbp: in_month.wbp.saturating_sub(seen.wbp),
            };
            let p = bill::project(&q, seen, remaining);
            (
                Some(bill::compute(&t.values, &q)),
                Some(bill::compute(&t.values, &p)),
            )
        }
        _ => (None, None),
    };
    Ok(MonthReport {
        site_id: site.id,
        site_name: site.name.clone(),
        month: month.to_string(),
        timezone: site.timezone.clone(),
        from,
        to,
        complete: now >= to,
        hours_in_month: in_month,
        hours_with_data: seen,
        quality,
        incomer_meters,
        tariff,
        month_to_date: mtd,
        projected,
        pln: None,
        reconciliation: None,
        confirmed: false,
        excluded: EXCLUDED.iter().map(|s| s.to_string()).collect(),
    })
}

impl MonthReport {
    /// Put PLN's figures for the month next to the shadow bill.
    pub fn attach_pln(&mut self, pln: PlnFigures) {
        if let (Some(b), Some(t)) = (&self.month_to_date, &self.tariff) {
            self.reconciliation = Some(bill::reconcile(b, &pln, &t.values));
        }
        self.pln = Some(pln);
    }
}

/// One local day of a month.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayRow {
    pub date: String,
    pub kwh_lwbp: f64,
    pub kwh_wbp: f64,
    pub kvarh: Option<f64>,
    pub cos_phi: Option<f64>,
    pub wbp_premium: f64,
    /// The day's own excess kVArh × rate. PLN bills the excess per month, so
    /// days do not add up to the month's charge exactly; this shows which
    /// days caused it.
    pub kvarh_charge: Option<f64>,
    pub cost_total: f64,
    pub quality: QualityCounts,
}

pub fn daily(site: &Site, tariffs: &[Tariff], month: &str, rows: &[HourRow]) -> Vec<DayRow> {
    let clock = site.clock();
    let Some(t) = tariff_for(tariffs, month) else {
        return Vec::new();
    };
    let mut by_day: BTreeMap<String, Vec<HourRow>> = BTreeMap::new();
    for r in rows {
        by_day
            .entry(format_date(clock.date(r.hour_start)))
            .or_default()
            .push(r.clone());
    }
    by_day
        .into_iter()
        .map(|(date, rows)| {
            let b = bill::compute(&t.values, &quantities(&rows));
            let mut quality = QualityCounts::default();
            for (_, q) in hour_qualities(&rows).values() {
                quality.add(*q);
            }
            DayRow {
                date,
                kwh_lwbp: b.kwh_lwbp,
                kwh_wbp: b.kwh_wbp,
                kvarh: b.kvarh,
                cos_phi: b.cos_phi,
                wbp_premium: b.wbp_premium,
                kvarh_charge: b.cost_kvarh,
                cost_total: b.total,
                quality,
            }
        })
        .collect()
}

/// Rolling power factor over `rows` (rows without kVArh are skipped).
pub fn rolling_pf(rows: &[HourRow]) -> Option<f64> {
    let (kwh, kvarh, n) = rows
        .iter()
        .filter_map(|r| r.kvarh.map(|k| (r.kwh, k)))
        .fold((0.0, 0.0, 0), |(a, b, n), (k, v)| (a + k, b + v, n + 1));
    if n == 0 {
        return None;
    }
    bill::cos_phi(kwh, kvarh)
}

/// Distinct hours in rows, for tests and checks.
pub fn distinct_hours(rows: &[HourRow]) -> BTreeSet<i64> {
    rows.iter().map(|r| r.hour_start).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::tariff::{AlertConfig, Golongan, TariffValues, WbpWindow};

    fn site() -> Site {
        Site {
            id: 1,
            name: "Plant".into(),
            timezone: "Asia/Jakarta".into(),
            wbp_windows: WbpWindow::default_windows(),
            contracted_kva: Some(2000.0),
            alerts: AlertConfig::default(),
            created_at: 0,
            updated_at: 0,
        }
    }

    fn tariff() -> Tariff {
        Tariff {
            id: 1,
            site_id: 1,
            valid_from: "2026-01".into(),
            values: TariffValues::defaults(Golongan::I3),
            note: String::new(),
        }
    }

    /// Every hour of a month at 100 kWh and 75 kVArh (cos φ 0.8).
    fn month_rows(site: &Site, month: &str, until: Option<i64>) -> Vec<HourRow> {
        let clock = site.clock();
        let (from, to) = clock.month_range(month).unwrap();
        (0..)
            .map(|i| from + i * 3600)
            .take_while(|h| *h < until.unwrap_or(to).min(to))
            .map(|h| HourRow {
                meter_id: 1,
                hour_start: h,
                kwh: 100.0,
                kvarh: Some(75.0),
                band: clock.band(h, &site.wbp_windows),
                quality: Quality::Ok,
            })
            .collect()
    }

    #[test]
    fn a_full_month_projects_to_itself() {
        let s = site();
        let rows = month_rows(&s, "2026-11", None);
        let r = month_report(&s, &[tariff()], "2026-11", &rows, vec![1], i64::MAX).unwrap();
        assert!(r.complete);
        assert_eq!(r.hours_in_month.total(), 720);
        assert_eq!(r.hours_in_month.wbp, 120);
        let (m, p) = (r.month_to_date.unwrap(), r.projected.unwrap());
        assert_eq!(m, p);
        assert!((m.kwh_wbp - 12_000.0).abs() < 1e-9);
        assert!((m.cos_phi.unwrap() - 0.8).abs() < 1e-12);
    }

    #[test]
    fn ten_days_project_to_the_month() {
        let s = site();
        let (from, _) = s.clock().month_range("2026-11").unwrap();
        let rows = month_rows(&s, "2026-11", Some(from + 10 * 86_400));
        let r = month_report(
            &s,
            &[tariff()],
            "2026-11",
            &rows,
            vec![1],
            from + 10 * 86_400,
        )
        .unwrap();
        assert!(!r.complete);
        let p = r.projected.unwrap();
        assert!((p.kwh_total - 72_000.0).abs() < 1e-6);
        assert!((p.kwh_wbp - 12_000.0).abs() < 1e-6);
        assert!((p.kvarh.unwrap() - 54_000.0).abs() < 1e-6);
        assert!((r.month_to_date.unwrap().kwh_total - 24_000.0).abs() < 1e-6);
    }

    #[test]
    fn daily_rows_follow_local_days() {
        let s = site();
        let rows = month_rows(&s, "2026-11", None);
        let days = daily(&s, &[tariff()], "2026-11", &rows);
        assert_eq!(days.len(), 30);
        assert_eq!(days[0].date, "2026-11-01");
        assert!((days[0].kwh_wbp - 400.0).abs() < 1e-9);
        // 400 kWh × 0.4 × 1035.78
        assert!((days[0].wbp_premium - 165_724.8).abs() < 0.01);
        assert_eq!(days[0].quality.ok, 24);
    }

    #[test]
    fn rolling_pf_skips_rows_without_kvarh() {
        let s = site();
        let mut rows = month_rows(&s, "2026-11", None);
        rows.truncate(3);
        rows[0].kvarh = None;
        assert!((rolling_pf(&rows).unwrap() - 0.8).abs() < 1e-12);
        rows.iter_mut().for_each(|r| r.kvarh = None);
        assert_eq!(rolling_pf(&rows), None);
    }
}
