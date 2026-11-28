//! The shadow PLN bill: energy lines, WBP premium and the excess-kVArh charge.
//!
//! * I-3: `cost = kWh_LWBP × T + kWh_WBP × K × T`; WBP premium `= kWh_WBP × (K − 1) × T`.
//! * I-4: one rate (`rate_wbp`, default equal to LWBP), no K.
//! * Excess kVArh, monthly: `max(0, kVArh − factor × kWh)` at the kVArh rate, with
//!   `factor = tan(acos(pf_limit))` unless configured. To be confirmed on a real bill.
//! * cos φ `= kWh / √(kWh² + kVArh²)`.
//!
//! Like the worksheet (`energy-research/pln-bill-worksheet.xlsx`), the bill leaves
//! out the demand charge (biaya beban), PPJ and PPN.

use serde::{Deserialize, Serialize};

use super::tariff::TariffValues;

/// Metered quantities of a period.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Quantities {
    pub kwh_lwbp: f64,
    pub kwh_wbp: f64,
    /// `None` when no meter reports reactive energy.
    pub kvarh: Option<f64>,
}

impl Quantities {
    pub fn kwh(&self) -> f64 {
        self.kwh_lwbp + self.kwh_wbp
    }

    pub fn add(&mut self, other: &Quantities) {
        self.kwh_lwbp += other.kwh_lwbp;
        self.kwh_wbp += other.kwh_wbp;
        self.kvarh = match (self.kvarh, other.kvarh) {
            (None, None) => None,
            (a, b) => Some(a.unwrap_or(0.0) + b.unwrap_or(0.0)),
        };
    }
}

/// Lines of a bill, Rupiah rounded to 2 decimals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bill {
    pub kwh_lwbp: f64,
    pub kwh_wbp: f64,
    pub kwh_total: f64,
    pub kvarh: Option<f64>,
    pub cos_phi: Option<f64>,
    pub rate_lwbp: f64,
    pub rate_wbp: f64,
    pub rate_kvarh: f64,
    pub k_factor: Option<f64>,
    pub kvarh_factor: f64,
    pub kvarh_allowed: f64,
    pub kvarh_excess: Option<f64>,
    pub cost_lwbp: f64,
    pub cost_wbp: f64,
    /// What WBP hours cost above the LWBP rate (included in `cost_wbp`).
    pub wbp_premium: f64,
    pub cost_kvarh: Option<f64>,
    /// `cost_lwbp + cost_wbp + cost_kvarh`, without demand charge, PPJ and PPN.
    pub total: f64,
}

pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub fn cos_phi(kwh: f64, kvarh: f64) -> Option<f64> {
    let s = (kwh * kwh + kvarh * kvarh).sqrt();
    (s > 0.0).then(|| kwh / s)
}

pub fn compute(t: &TariffValues, q: &Quantities) -> Bill {
    let kwh = q.kwh();
    let rate_wbp = t.wbp_rate();
    let factor = t.excess_factor();
    let allowed = factor * kwh;
    let excess = q.kvarh.map(|kvarh| (kvarh - allowed).max(0.0));
    let cost_lwbp = q.kwh_lwbp * t.rate_lwbp;
    let cost_wbp = q.kwh_wbp * rate_wbp;
    let cost_kvarh = excess.map(|e| e * t.rate_kvarh);
    Bill {
        kwh_lwbp: q.kwh_lwbp,
        kwh_wbp: q.kwh_wbp,
        kwh_total: kwh,
        kvarh: q.kvarh,
        cos_phi: q.kvarh.and_then(|k| cos_phi(kwh, k)),
        rate_lwbp: t.rate_lwbp,
        rate_wbp,
        rate_kvarh: t.rate_kvarh,
        k_factor: t.k(),
        kvarh_factor: factor,
        kvarh_allowed: allowed,
        kvarh_excess: excess,
        cost_lwbp: round2(cost_lwbp),
        cost_wbp: round2(cost_wbp),
        wbp_premium: round2(q.kwh_wbp * (rate_wbp - t.rate_lwbp)),
        cost_kvarh: cost_kvarh.map(round2),
        total: round2(cost_lwbp + cost_wbp + cost_kvarh.unwrap_or(0.0)),
    }
}

/// Month-end quantities from month-to-date ones: each band's rate per hour so
/// far, times the hours of that band left in the month.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct HourCounts {
    pub lwbp: u32,
    pub wbp: u32,
}

impl HourCounts {
    pub fn total(&self) -> u32 {
        self.lwbp + self.wbp
    }
}

pub fn project(mtd: &Quantities, seen: HourCounts, remaining: HourCounts) -> Quantities {
    let per = |v: f64, n: u32| if n > 0 { v / n as f64 } else { 0.0 };
    Quantities {
        kwh_lwbp: mtd.kwh_lwbp + per(mtd.kwh_lwbp, seen.lwbp) * remaining.lwbp as f64,
        kwh_wbp: mtd.kwh_wbp + per(mtd.kwh_wbp, seen.wbp) * remaining.wbp as f64,
        kvarh: mtd
            .kvarh
            .map(|k| k + per(k, seen.total()) * remaining.total() as f64),
    }
}

// ── Reconciliation against PLN's bill ──

/// Figures copied from PLN's bill for one month. Quantities are required;
/// Rupiah lines are optional (the total is then priced from PLN's quantities
/// with the site's tariff).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlnFigures {
    pub kwh_lwbp: f64,
    pub kwh_wbp: f64,
    pub kvarh: f64,
    #[serde(default)]
    pub kvarh_excess: Option<f64>,
    #[serde(default)]
    pub cost_lwbp: Option<f64>,
    #[serde(default)]
    pub cost_wbp: Option<f64>,
    #[serde(default)]
    pub cost_kvarh: Option<f64>,
    /// LWBP + WBP + kVArh charge in Rupiah, without demand charge, PPJ and PPN.
    #[serde(default)]
    pub total: Option<f64>,
    #[serde(default)]
    pub note: String,
}

impl PlnFigures {
    pub fn validate(&self) -> Result<(), String> {
        let ok = |v: f64| v.is_finite() && v >= 0.0;
        if ![self.kwh_lwbp, self.kwh_wbp, self.kvarh]
            .into_iter()
            .all(ok)
        {
            return Err("kWh and kVArh must be zero or more".into());
        }
        for v in [
            self.kvarh_excess,
            self.cost_lwbp,
            self.cost_wbp,
            self.cost_kvarh,
            self.total,
        ]
        .into_iter()
        .flatten()
        {
            if !ok(v) {
                return Err("bill amounts must be zero or more".into());
            }
        }
        if self.note.len() > 2000 {
            return Err("note is too long".into());
        }
        Ok(())
    }

    /// PLN's total: as printed, else the sum of its Rupiah lines when all are
    /// given, else PLN's quantities priced with `t`.
    pub fn total_idr(&self, t: &TariffValues) -> f64 {
        if let Some(total) = self.total {
            return total;
        }
        if let (Some(a), Some(b), Some(c)) = (self.cost_lwbp, self.cost_wbp, self.cost_kvarh) {
            return a + b + c;
        }
        compute(
            t,
            &Quantities {
                kwh_lwbp: self.kwh_lwbp,
                kwh_wbp: self.kwh_wbp,
                kvarh: Some(self.kvarh),
            },
        )
        .total
    }
}

/// One compared line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diff {
    pub line: String,
    pub computed: Option<f64>,
    pub pln: f64,
    /// computed − PLN.
    pub difference: Option<f64>,
    /// difference / PLN; `None` when PLN's figure is zero.
    pub difference_pct: Option<f64>,
}

/// A month's shadow bill next to PLN's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reconciliation {
    pub lines: Vec<Diff>,
    /// Whether the totals agree within [`RECONCILE_TOLERANCE`].
    pub within_tolerance: bool,
}

/// Two percent: below it the tariff setup counts as confirmed (spec §7).
pub const RECONCILE_TOLERANCE: f64 = 0.02;

pub fn reconcile(computed: &Bill, pln: &PlnFigures, t: &TariffValues) -> Reconciliation {
    let diff = |line: &str, c: Option<f64>, p: f64| {
        let d = c.map(|c| c - p);
        Diff {
            line: line.into(),
            computed: c,
            pln: p,
            difference: d,
            difference_pct: d.and_then(|d| (p != 0.0).then(|| d / p)),
        }
    };
    let mut lines = vec![
        diff("kwh_lwbp", Some(computed.kwh_lwbp), pln.kwh_lwbp),
        diff("kwh_wbp", Some(computed.kwh_wbp), pln.kwh_wbp),
        diff("kvarh", computed.kvarh, pln.kvarh),
    ];
    if let Some(p) = pln.kvarh_excess {
        lines.push(diff("kvarh_excess", computed.kvarh_excess, p));
    }
    if let Some(p) = pln.cost_lwbp {
        lines.push(diff("cost_lwbp", Some(computed.cost_lwbp), p));
    }
    if let Some(p) = pln.cost_wbp {
        lines.push(diff("cost_wbp", Some(computed.cost_wbp), p));
    }
    if let Some(p) = pln.cost_kvarh {
        lines.push(diff("cost_kvarh", computed.cost_kvarh, p));
    }
    let total = diff("total", Some(computed.total), pln.total_idr(t));
    let within_tolerance = total
        .difference_pct
        .is_some_and(|p| p.abs() <= RECONCILE_TOLERANCE)
        || (total.pln == 0.0 && computed.total == 0.0);
    lines.push(total);
    Reconciliation {
        lines,
        within_tolerance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::tariff::{Golongan, TariffValues};

    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() <= tol, "{a} != {b}");
    }

    /// The worksheet's illustration: 600,000 kWh, 20% in WBP, K = 1.4, cos φ 0.80.
    fn illustration() -> Quantities {
        Quantities {
            kwh_lwbp: 480_000.0,
            kwh_wbp: 120_000.0,
            kvarh: Some(450_000.0),
        }
    }

    #[test]
    fn worksheet_illustration_with_the_rounded_062_factor() {
        // Spec §2 and the research round tan(acos 0.85) to 0.62.
        let mut t = TariffValues::defaults(Golongan::I3);
        t.kvarh_factor = Some(0.62);
        let b = compute(&t, &illustration());
        close(b.kvarh_excess.unwrap(), 78_000.0, 1e-6);
        close(b.cost_kvarh.unwrap(), 86_949_720.0, 0.005);
        close(b.wbp_premium, 49_717_440.0, 0.005);
        close(b.cos_phi.unwrap(), 0.8, 1e-12);
    }

    #[test]
    fn worksheet_illustration_with_the_exact_factor_matches_its_cells() {
        // pln-bill-worksheet.xlsx computes TAN(ACOS(0.85)) exactly (cells B24-B31).
        let b = compute(&TariffValues::defaults(Golongan::I3), &illustration());
        close(b.cost_lwbp, 497_174_400.0, 0.005);
        close(b.cost_wbp, 174_011_040.0, 0.005);
        close(b.wbp_premium, 49_717_440.0, 0.005);
        close(b.kvarh_allowed, 371_846.603_041_861_4, 1e-6);
        close(b.kvarh_excess.unwrap(), 78_153.396_958_138_62, 1e-6);
        close(b.cost_kvarh.unwrap(), 87_120_717.73, 0.005);
        close(b.total, 758_306_157.73, 0.005);
    }

    #[test]
    fn i4_has_one_rate_and_no_premium() {
        let t = TariffValues::defaults(Golongan::I4);
        let b = compute(&t, &illustration());
        assert_eq!(b.k_factor, None);
        close(b.cost_lwbp, 480_000.0 * 996.74, 0.005);
        close(b.cost_wbp, 120_000.0 * 996.74, 0.005);
        assert_eq!(b.wbp_premium, 0.0);
        close(b.cost_kvarh.unwrap(), 78_153.396_958_138_62 * 996.74, 0.01);
    }

    #[test]
    fn good_power_factor_has_no_kvarh_charge() {
        let b = compute(
            &TariffValues::defaults(Golongan::I3),
            &Quantities {
                kwh_lwbp: 1000.0,
                kwh_wbp: 0.0,
                kvarh: Some(300.0),
            },
        );
        assert_eq!(b.kvarh_excess, Some(0.0));
        assert_eq!(b.cost_kvarh, Some(0.0));
        let no_kvarh = compute(
            &TariffValues::defaults(Golongan::I3),
            &Quantities {
                kwh_lwbp: 1000.0,
                kwh_wbp: 0.0,
                kvarh: None,
            },
        );
        assert_eq!(no_kvarh.cos_phi, None);
        assert_eq!(no_kvarh.cost_kvarh, None);
    }

    #[test]
    fn projection_scales_each_band() {
        let mtd = Quantities {
            kwh_lwbp: 200.0,
            kwh_wbp: 40.0,
            kvarh: Some(120.0),
        };
        let p = project(
            &mtd,
            HourCounts { lwbp: 20, wbp: 4 },
            HourCounts { lwbp: 40, wbp: 8 },
        );
        close(p.kwh_lwbp, 600.0, 1e-9);
        close(p.kwh_wbp, 120.0, 1e-9);
        close(p.kvarh.unwrap(), 360.0, 1e-9);
    }

    #[test]
    fn reconcile_reports_differences_and_the_2pct_rule() {
        let t = TariffValues::defaults(Golongan::I3);
        let ours = compute(&t, &illustration());
        let pln = PlnFigures {
            kwh_lwbp: 485_000.0,
            kwh_wbp: 121_000.0,
            kvarh: 452_000.0,
            kvarh_excess: None,
            cost_lwbp: None,
            cost_wbp: None,
            cost_kvarh: None,
            total: None,
            note: String::new(),
        };
        let r = reconcile(&ours, &pln, &t);
        let kwh = r.lines.iter().find(|l| l.line == "kwh_lwbp").unwrap();
        close(kwh.difference.unwrap(), -5_000.0, 1e-9);
        assert!(r.within_tolerance, "{r:?}");
        let off = PlnFigures {
            total: Some(ours.total * 1.05),
            ..pln
        };
        assert!(!reconcile(&ours, &off, &t).within_tolerance);
    }
}
