//! Sites, PLN tariffs and the WBP (peak) clock.
//!
//! Tariff values are data: the defaults below are the 2025 PLN rates from the
//! energy research (`energy-research/README.md`), offered when a site is created
//! and editable afterwards. A tariff applies from the first day of its
//! `valid_from` month, so a rate change is a new row, not an edit.

use serde::{Deserialize, Serialize};
use time::{Date, Month, UtcOffset};

/// PLN tariff group (golongan) the bill shadow supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Golongan {
    /// I-3/TM, above 200 kVA: WBP energy costs K times the LWBP rate.
    #[serde(rename = "I-3")]
    I3,
    /// I-4/TT, 30,000 kVA and above: one rate, no K.
    #[serde(rename = "I-4")]
    I4,
}

impl Golongan {
    pub fn as_str(self) -> &'static str {
        match self {
            Golongan::I3 => "I-3",
            Golongan::I4 => "I-4",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "I-3" | "I3" | "i-3" | "i3" => Some(Golongan::I3),
            "I-4" | "I4" | "i-4" | "i4" => Some(Golongan::I4),
            _ => None,
        }
    }
}

/// PLN's power factor limit; reactive energy above it is billed.
pub const DEFAULT_PF_LIMIT: f64 = 0.85;

/// The tariff values a bill is computed with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TariffValues {
    pub golongan: Golongan,
    /// WBP multiplier for I-3 (1.4 ≤ K ≤ 2, printed on the bill). Ignored for I-4.
    #[serde(default)]
    pub k_factor: Option<f64>,
    /// Rp/kWh outside peak hours (and the single rate of I-4).
    pub rate_lwbp: f64,
    /// Rp/kWh in peak hours, I-4 only (defaults to `rate_lwbp`). For I-3 the WBP
    /// rate is always K × `rate_lwbp`.
    #[serde(default)]
    pub rate_wbp: Option<f64>,
    /// Rp per excess kVArh.
    pub rate_kvarh: f64,
    /// Monthly power factor below which reactive energy is billed.
    #[serde(default = "default_pf_limit")]
    pub pf_limit: f64,
    /// kVArh allowed per kWh before the excess is billed. `None` uses
    /// tan(acos(`pf_limit`)) ≈ 0.6197 at 0.85, as the worksheet does. To be
    /// confirmed against the tariff regulation and a real bill.
    #[serde(default)]
    pub kvarh_factor: Option<f64>,
}

fn default_pf_limit() -> f64 {
    DEFAULT_PF_LIMIT
}

impl TariffValues {
    /// PLN 2025 rates for a tariff group (research §1.2): I-3 Rp 1,035.78/kWh,
    /// K 1.4, kVArh Rp 1,114.74; I-4 Rp 996.74 for every line.
    pub fn defaults(golongan: Golongan) -> Self {
        match golongan {
            Golongan::I3 => TariffValues {
                golongan,
                k_factor: Some(1.4),
                rate_lwbp: 1035.78,
                rate_wbp: None,
                rate_kvarh: 1114.74,
                pf_limit: DEFAULT_PF_LIMIT,
                kvarh_factor: None,
            },
            Golongan::I4 => TariffValues {
                golongan,
                k_factor: None,
                rate_lwbp: 996.74,
                rate_wbp: Some(996.74),
                rate_kvarh: 996.74,
                pf_limit: DEFAULT_PF_LIMIT,
                kvarh_factor: None,
            },
        }
    }

    /// K as used in the bill: the configured one for I-3 (1.4 if missing), none for I-4.
    pub fn k(&self) -> Option<f64> {
        match self.golongan {
            Golongan::I3 => Some(self.k_factor.unwrap_or(1.4)),
            Golongan::I4 => None,
        }
    }

    /// Rp/kWh charged in WBP hours.
    pub fn wbp_rate(&self) -> f64 {
        match self.golongan {
            Golongan::I3 => self.k().unwrap_or(1.0) * self.rate_lwbp,
            Golongan::I4 => self.rate_wbp.unwrap_or(self.rate_lwbp),
        }
    }

    /// kVArh allowed per kWh: the configured factor, else tan(acos(pf_limit)).
    pub fn excess_factor(&self) -> f64 {
        self.kvarh_factor
            .unwrap_or_else(|| self.pf_limit.acos().tan())
    }

    pub fn validate(&self) -> Result<(), String> {
        let rate_ok = |v: f64| v.is_finite() && v > 0.0 && v < 1_000_000.0;
        if !rate_ok(self.rate_lwbp) {
            return Err("rate_lwbp must be a positive Rp/kWh rate".into());
        }
        if !rate_ok(self.rate_kvarh) {
            return Err("rate_kvarh must be a positive Rp/kVArh rate".into());
        }
        if let Some(w) = self.rate_wbp
            && !rate_ok(w)
        {
            return Err("rate_wbp must be a positive Rp/kWh rate".into());
        }
        if self.golongan == Golongan::I3 {
            match self.k_factor {
                Some(k) if (1.4..=2.0).contains(&k) => {}
                Some(_) => return Err("K must be between 1.4 and 2 for I-3".into()),
                None => return Err("K is required for I-3".into()),
            }
        }
        if !(self.pf_limit > 0.5 && self.pf_limit < 1.0) {
            return Err("pf_limit must be between 0.5 and 1".into());
        }
        if let Some(f) = self.kvarh_factor
            && !(f.is_finite() && (0.0..=2.0).contains(&f))
        {
            return Err("kvarh_factor must be between 0 and 2".into());
        }
        Ok(())
    }
}

/// A tariff row: the values plus the billing month it starts in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tariff {
    pub id: i64,
    pub site_id: i64,
    /// First billing month (`YYYY-MM`, local time) the values apply to.
    pub valid_from: String,
    #[serde(flatten)]
    pub values: TariffValues,
    #[serde(default)]
    pub note: String,
}

/// The tariff in force for `month`: the latest one starting on or before it,
/// else the earliest one (a month before the first tariff is priced with it).
pub fn tariff_for<'a>(tariffs: &'a [Tariff], month: &str) -> Option<&'a Tariff> {
    tariffs
        .iter()
        .filter(|t| t.valid_from.as_str() <= month)
        .max_by(|a, b| a.valid_from.cmp(&b.valid_from))
        .or_else(|| {
            tariffs
                .iter()
                .min_by(|a, b| a.valid_from.cmp(&b.valid_from))
        })
}

// ── WBP windows and local time ──

/// A WBP (peak) window in the site's local time, whole hours. `end` may be
/// before `start` for a window across midnight; `end` "24:00" means midnight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WbpWindow {
    pub start: String,
    pub end: String,
}

impl WbpWindow {
    pub fn default_windows() -> Vec<WbpWindow> {
        vec![WbpWindow {
            start: "18:00".into(),
            end: "22:00".into(),
        }]
    }

    fn hours(&self) -> Result<(u32, u32), String> {
        Ok((parse_hour(&self.start)?, parse_hour(&self.end)?))
    }
}

/// `HH:00` to an hour of the day (0-24). Minutes must be 00: bills and
/// `energy_hour` rows are per whole hour.
fn parse_hour(s: &str) -> Result<u32, String> {
    let (h, m) = s
        .trim()
        .split_once(':')
        .ok_or_else(|| format!("'{s}' is not HH:MM"))?;
    let h: u32 = h.parse().map_err(|_| format!("'{s}' is not HH:MM"))?;
    let m: u32 = m.parse().map_err(|_| format!("'{s}' is not HH:MM"))?;
    if m != 0 {
        return Err(format!("'{s}': WBP windows start and end on whole hours"));
    }
    if h > 24 {
        return Err(format!("'{s}' is not a time of day"));
    }
    Ok(h)
}

pub fn validate_windows(windows: &[WbpWindow]) -> Result<(), String> {
    if windows.len() > 8 {
        return Err("at most 8 WBP windows".into());
    }
    for w in windows {
        let (s, e) = w.hours()?;
        if s % 24 == e % 24 {
            return Err(format!("WBP window {}-{} is empty", w.start, w.end));
        }
    }
    Ok(())
}

/// Whether the local hour of day `h` (0-23) lies in a WBP window.
pub fn is_wbp_hour(h: u32, windows: &[WbpWindow]) -> bool {
    windows.iter().any(|w| match w.hours() {
        Ok((s, e)) => {
            let (s, e) = (s % 24, e % 24);
            if s < e {
                h >= s && h < e
            } else {
                h >= s || h < e
            }
        }
        Err(_) => false,
    })
}

/// LWBP (off-peak) or WBP (peak).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Band {
    #[serde(rename = "LWBP")]
    Lwbp,
    #[serde(rename = "WBP")]
    Wbp,
}

impl Band {
    pub fn as_str(self) -> &'static str {
        match self {
            Band::Lwbp => "LWBP",
            Band::Wbp => "WBP",
        }
    }

    pub fn parse(s: &str) -> Self {
        if s == "WBP" { Band::Wbp } else { Band::Lwbp }
    }
}

/// The local clock of a site: a fixed UTC offset in whole hours. Indonesia has
/// no daylight saving, so WIB/WITA/WIT are fixed offsets and the UTC hours of
/// `energy_hour` are also whole local hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalClock {
    pub offset_secs: i64,
}

impl LocalClock {
    /// `Asia/Jakarta`, `Asia/Pontianak` (WIB, +7), `Asia/Makassar` (WITA, +8),
    /// `Asia/Jayapura` (WIT, +9), `UTC`, or a fixed offset like `+07:00` / `UTC+7`.
    pub fn parse(tz: &str) -> Result<Self, String> {
        let t = tz.trim();
        let hours = match t {
            "Asia/Jakarta" | "Asia/Pontianak" | "WIB" => 7,
            "Asia/Makassar" | "Asia/Ujung_Pandang" | "WITA" => 8,
            "Asia/Jayapura" | "WIT" => 9,
            "UTC" | "Etc/UTC" | "Z" => 0,
            other => {
                let s = other.strip_prefix("UTC").unwrap_or(other);
                let (sign, rest) = match s.as_bytes().first() {
                    Some(b'+') => (1, &s[1..]),
                    Some(b'-') => (-1, &s[1..]),
                    _ => return Err(format!("unknown timezone '{tz}'")),
                };
                let (h, m) = rest.split_once(':').unwrap_or((rest, "0"));
                let h: i64 = h.parse().map_err(|_| format!("unknown timezone '{tz}'"))?;
                let m: i64 = m.parse().map_err(|_| format!("unknown timezone '{tz}'"))?;
                if m != 0 || h > 14 {
                    return Err(format!(
                        "timezone '{tz}': only whole-hour offsets are supported"
                    ));
                }
                sign * h
            }
        };
        Ok(LocalClock {
            offset_secs: hours * 3600,
        })
    }

    pub fn utc_offset(&self) -> UtcOffset {
        UtcOffset::from_whole_seconds(self.offset_secs as i32).unwrap_or(UtcOffset::UTC)
    }

    /// Local hour of day (0-23) of a UTC instant.
    pub fn hour_of_day(&self, ts: i64) -> u32 {
        ((ts + self.offset_secs).rem_euclid(86_400) / 3600) as u32
    }

    /// Local calendar day of a UTC instant.
    pub fn date(&self, ts: i64) -> Date {
        let days = (ts + self.offset_secs).div_euclid(86_400);
        Date::from_julian_day((days + UNIX_EPOCH_JULIAN) as i32).unwrap_or(Date::MIN)
    }

    /// UTC instant of local midnight starting `date`.
    pub fn midnight(&self, date: Date) -> i64 {
        (date.to_julian_day() as i64 - UNIX_EPOCH_JULIAN) * 86_400 - self.offset_secs
    }

    /// `[from, to)` in UTC seconds of a local billing month `YYYY-MM`.
    pub fn month_range(&self, month: &str) -> Result<(i64, i64), String> {
        let (y, m) = parse_month(month)?;
        let first = Date::from_calendar_date(y, m, 1).map_err(|e| e.to_string())?;
        let next = if m == Month::December {
            Date::from_calendar_date(y + 1, Month::January, 1)
        } else {
            Date::from_calendar_date(y, m.next(), 1)
        }
        .map_err(|e| e.to_string())?;
        Ok((self.midnight(first), self.midnight(next)))
    }

    /// The local billing month (`YYYY-MM`) of a UTC instant.
    pub fn month_of(&self, ts: i64) -> String {
        let d = self.date(ts);
        format!("{:04}-{:02}", d.year(), d.month() as u8)
    }

    pub fn band(&self, hour_start: i64, windows: &[WbpWindow]) -> Band {
        if is_wbp_hour(self.hour_of_day(hour_start), windows) {
            Band::Wbp
        } else {
            Band::Lwbp
        }
    }
}

/// Julian day number of 1970-01-01.
const UNIX_EPOCH_JULIAN: i64 = 2_440_588;

/// `YYYY-MM` to year and month.
pub fn parse_month(s: &str) -> Result<(i32, Month), String> {
    let bad = || format!("'{s}' is not a month (YYYY-MM)");
    let (y, m) = s.trim().split_once('-').ok_or_else(bad)?;
    if y.len() != 4 || m.len() != 2 {
        return Err(bad());
    }
    let y: i32 = y.parse().map_err(|_| bad())?;
    let m: u8 = m.parse().map_err(|_| bad())?;
    let m = Month::try_from(m).map_err(|_| bad())?;
    if !(2000..=2100).contains(&y) {
        return Err(bad());
    }
    Ok((y, m))
}

pub fn format_date(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
}

// ── Sites ──

/// Alert settings of a site (spec 3.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertConfig {
    /// Warn when the rolling power factor of the PLN incomers drops below this
    /// (an early warning before PLN's 0.85).
    pub pf_warn: f64,
    /// Hours the rolling power factor covers.
    pub pf_window_hours: u32,
    /// Warn when the average load in a WBP hour exceeds this many kW. `None`: off.
    pub wbp_kw_target: Option<f64>,
    /// Send yesterday's kVArh charge and WBP premium once a day through the
    /// notification rules of source `energy:site<id>:daily`.
    pub daily_summary: bool,
    /// Language of the daily summary: `id` or `en`.
    pub lang: String,
}

impl Default for AlertConfig {
    fn default() -> Self {
        AlertConfig {
            pf_warn: 0.87,
            pf_window_hours: 24,
            wbp_kw_target: None,
            daily_summary: true,
            lang: "id".into(),
        }
    }
}

impl AlertConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !(self.pf_warn > 0.0 && self.pf_warn < 1.0) {
            return Err("pf_warn must be between 0 and 1".into());
        }
        if !(1..=24 * 31).contains(&self.pf_window_hours) {
            return Err("pf_window_hours must be 1 to 744".into());
        }
        if let Some(kw) = self.wbp_kw_target
            && !(kw.is_finite() && kw > 0.0)
        {
            return Err("wbp_kw_target must be a positive kW value".into());
        }
        if !matches!(self.lang.as_str(), "id" | "en") {
            return Err("lang must be 'id' or 'en'".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub timezone: String,
    pub wbp_windows: Vec<WbpWindow>,
    /// Contracted power (daya tersambung), kVA. For reference: the demand
    /// charge it drives is not part of the shadow bill.
    pub contracted_kva: Option<f64>,
    pub alerts: AlertConfig,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Site {
    pub fn clock(&self) -> LocalClock {
        LocalClock::parse(&self.timezone).unwrap_or(LocalClock {
            offset_secs: 7 * 3600,
        })
    }
}

pub const DEFAULT_TIMEZONE: &str = "Asia/Jakarta";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_excess_factor_is_tan_acos_085() {
        let t = TariffValues::defaults(Golongan::I3);
        assert!((t.excess_factor() - 0.619_744).abs() < 1e-6);
        assert!((t.wbp_rate() - 1450.092).abs() < 1e-9);
        let i4 = TariffValues::defaults(Golongan::I4);
        assert_eq!(i4.k(), None);
        assert_eq!(i4.wbp_rate(), 996.74);
        assert!(t.validate().is_ok() && i4.validate().is_ok());
    }

    #[test]
    fn k_outside_the_regulated_range_is_refused() {
        let mut t = TariffValues::defaults(Golongan::I3);
        t.k_factor = Some(2.5);
        assert!(t.validate().is_err());
        t.k_factor = Some(1.0);
        assert!(t.validate().is_err());
        t.k_factor = None;
        assert!(t.validate().is_err());
    }

    #[test]
    fn wbp_banding_in_wib_and_across_midnight() {
        let wib = LocalClock::parse("Asia/Jakarta").unwrap();
        let w = WbpWindow::default_windows();
        // 2026-11-02 11:00 UTC = 18:00 WIB: first WBP hour.
        let h18 =
            wib.midnight(Date::from_calendar_date(2026, Month::November, 2).unwrap()) + 18 * 3600;
        assert_eq!(h18.rem_euclid(86_400), 11 * 3600);
        assert_eq!(wib.band(h18 - 3600, &w), Band::Lwbp);
        assert_eq!(wib.band(h18, &w), Band::Wbp);
        assert_eq!(wib.band(h18 + 3 * 3600, &w), Band::Wbp);
        assert_eq!(wib.band(h18 + 4 * 3600, &w), Band::Lwbp);
        // The same UTC hour is 19:00 WITA (still WBP) and 20:00 WIT.
        let wita = LocalClock::parse("Asia/Makassar").unwrap();
        assert_eq!(wita.hour_of_day(h18), 19);
        assert_eq!(wita.band(h18 + 3 * 3600, &w), Band::Lwbp);

        // A window across midnight, 22:00-02:00.
        let night = vec![WbpWindow {
            start: "22:00".into(),
            end: "02:00".into(),
        }];
        let midnight = wib.midnight(Date::from_calendar_date(2026, Month::November, 3).unwrap());
        assert_eq!(wib.band(midnight - 3 * 3600, &night), Band::Lwbp); // 21:00
        assert_eq!(wib.band(midnight - 2 * 3600, &night), Band::Wbp); // 22:00
        assert_eq!(wib.band(midnight, &night), Band::Wbp); // 00:00
        assert_eq!(wib.band(midnight + 3600, &night), Band::Wbp); // 01:00
        assert_eq!(wib.band(midnight + 2 * 3600, &night), Band::Lwbp); // 02:00
        // The local day of 23:00 WIB is the evening's date, though UTC is 16:00.
        assert_eq!(format_date(wib.date(midnight - 3600)), "2026-11-02");
    }

    #[test]
    fn windows_must_be_whole_hours_and_non_empty() {
        let w = |s: &str, e: &str| {
            vec![WbpWindow {
                start: s.into(),
                end: e.into(),
            }]
        };
        assert!(validate_windows(&w("18:00", "22:00")).is_ok());
        assert!(validate_windows(&w("17:30", "22:00")).is_err());
        assert!(validate_windows(&w("18:00", "18:00")).is_err());
        assert!(validate_windows(&w("22:00", "24:00")).is_ok());
        assert!(is_wbp_hour(23, &w("22:00", "24:00")));
        assert!(!is_wbp_hour(0, &w("22:00", "24:00")));
    }

    #[test]
    fn month_range_is_local_midnight() {
        let wib = LocalClock::parse("+07:00").unwrap();
        let (from, to) = wib.month_range("2026-11").unwrap();
        assert_eq!(to - from, 30 * 86_400);
        assert_eq!(wib.hour_of_day(from), 0);
        assert_eq!(wib.month_of(from), "2026-11");
        assert_eq!(wib.month_of(from - 1), "2026-10");
        let (_, dec_end) = wib.month_range("2026-12").unwrap();
        assert_eq!(wib.month_of(dec_end), "2027-01");
        assert!(LocalClock::parse("+05:30").is_err());
        assert!(LocalClock::parse("Mars/Base").is_err());
        assert!(parse_month("2026-13").is_err());
    }

    #[test]
    fn the_tariff_of_a_month_is_the_latest_started() {
        let t = |id, from: &str| Tariff {
            id,
            site_id: 1,
            valid_from: from.into(),
            values: TariffValues::defaults(Golongan::I3),
            note: String::new(),
        };
        let all = vec![t(1, "2025-01"), t(2, "2026-07")];
        assert_eq!(tariff_for(&all, "2026-06").unwrap().id, 1);
        assert_eq!(tariff_for(&all, "2026-07").unwrap().id, 2);
        assert_eq!(tariff_for(&all, "2024-03").unwrap().id, 1);
        assert!(tariff_for(&[], "2026-07").is_none());
    }
}
