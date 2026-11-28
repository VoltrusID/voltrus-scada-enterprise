//! Hourly energy from cumulative meter counters.
//!
//! `ΔkWh_h = E(end of h) − E(start of h)`. The samples are first made monotonic
//! ([`CounterUnwrapper`]: rollover adds the rollover value, a reset adds
//! nothing), then the counter value at each hour boundary is read from the
//! samples on either side of it, interpolated linearly. Energy is conserved:
//! the hours of a gap share the counter difference across the gap.
//!
//! Quality per hour:
//! * `ok`: samples within `max_gap` seconds on both sides of both boundaries.
//! * `estimated`: a boundary was interpolated across a longer gap, or the
//!   counter reset during the hour (the energy of that one step is lost).
//! * `gap`: a boundary has no sample after it (or none before it): the hour
//!   holds what the samples inside it show, usually less than was used. The
//!   hourly job computes such hours again when data arrives.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    Ok,
    Estimated,
    Gap,
}

impl Quality {
    pub fn as_str(self) -> &'static str {
        match self {
            Quality::Ok => "ok",
            Quality::Estimated => "estimated",
            Quality::Gap => "gap",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "ok" => Quality::Ok,
            "estimated" => Quality::Estimated,
            _ => Quality::Gap,
        }
    }

    pub fn worst(self, other: Quality) -> Quality {
        self.max(other)
    }
}

/// A sample of a monotonic (unwrapped) counter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub ts: i64,
    pub value: f64,
    /// The counter went down here without a rollover: the step into this
    /// sample counted as zero.
    pub reset: bool,
}

/// Makes raw counter readings monotonic.
#[derive(Debug, Clone)]
pub struct CounterUnwrapper {
    rollover: Option<f64>,
    offset: f64,
    prev_raw: Option<f64>,
    last: f64,
}

impl CounterUnwrapper {
    /// `rollover`: the value the counter wraps at (for example 4294967296 for a
    /// 32-bit register), in raw counter units.
    pub fn new(rollover: Option<f64>) -> Self {
        CounterUnwrapper {
            rollover: rollover.filter(|r| r.is_finite() && *r > 0.0),
            offset: 0.0,
            prev_raw: None,
            last: 0.0,
        }
    }

    pub fn push(&mut self, ts: i64, raw: f64) -> Option<Point> {
        if !raw.is_finite() {
            return None;
        }
        let mut reset = false;
        let value = match self.prev_raw {
            None => {
                self.prev_raw = Some(raw);
                raw + self.offset
            }
            Some(p) => {
                let d = raw - p;
                // Float registers can wobble in the last digit; that is not a reset.
                let tolerance = 1e-6 * p.abs().max(1.0);
                if d >= 0.0 {
                    self.prev_raw = Some(raw);
                    raw + self.offset
                } else if -d <= tolerance {
                    self.last
                } else if let Some(r) = self.rollover
                    && p >= 0.9 * r
                    && raw <= 0.1 * r
                {
                    self.offset += r;
                    self.prev_raw = Some(raw);
                    raw + self.offset
                } else {
                    // Reset (cleared or replaced meter): continue from the last
                    // value so the drop is neither negative energy nor a spike.
                    reset = true;
                    self.offset = self.last - raw;
                    self.prev_raw = Some(raw);
                    self.last
                }
            }
        };
        self.last = value;
        Some(Point { ts, value, reset })
    }
}

/// Turns a power reading (kvar, kW) into a cumulative energy series with the
/// trapezoid rule. Steps longer than `max_gap` seconds add nothing: the hours
/// across them come out `estimated` with no energy.
#[derive(Debug, Clone)]
pub struct RateIntegrator {
    max_gap: i64,
    last: Option<(i64, f64)>,
    total: f64,
}

impl RateIntegrator {
    pub fn new(max_gap: i64) -> Self {
        RateIntegrator {
            max_gap,
            last: None,
            total: 0.0,
        }
    }

    pub fn push(&mut self, ts: i64, rate: f64) -> Option<Point> {
        if !rate.is_finite() {
            return None;
        }
        if let Some((lt, lv)) = self.last {
            let dt = ts - lt;
            if dt > 0 && dt <= self.max_gap {
                self.total += (lv + rate) / 2.0 * dt as f64 / 3600.0;
            }
        }
        self.last = Some((ts, rate));
        Some(Point {
            ts,
            value: self.total,
            reset: false,
        })
    }
}

/// Keep what [`hourly_deltas`] needs from one hour's points: the first, the
/// last and every reset.
pub fn compact(points: Vec<Point>) -> Vec<Point> {
    let n = points.len();
    if n <= 2 {
        return points;
    }
    points
        .into_iter()
        .enumerate()
        .filter(|(i, p)| *i == 0 || *i == n - 1 || p.reset)
        .map(|(_, p)| p)
        .collect()
}

/// Energy of one hour, in counter units (before the CT/PT multiplier).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HourDelta {
    pub hour_start: i64,
    pub delta: f64,
    pub quality: Quality,
}

/// The counter at `at`: `(value, span)` where `span` is the distance between
/// the samples it was read from (0 when a sample sits on `at`).
fn value_at(points: &[Point], at: i64) -> Option<(f64, i64)> {
    let idx = points.partition_point(|p| p.ts < at);
    if let Some(p) = points.get(idx)
        && p.ts == at
    {
        return Some((p.value, 0));
    }
    if idx == 0 || idx >= points.len() {
        return None;
    }
    let (a, b) = (points[idx - 1], points[idx]);
    let span = b.ts - a.ts;
    let f = (at - a.ts) as f64 / span as f64;
    Some((a.value + (b.value - a.value) * f, span))
}

/// Per-hour deltas for the hours starting in `[from, to)` (hour-aligned), from
/// monotonic `points` sorted by time. Hours that end before the first point
/// are left out: there is nothing to say about them.
pub fn hourly_deltas(points: &[Point], from: i64, to: i64, max_gap: i64) -> Vec<HourDelta> {
    let Some(first) = points.first() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut s = from;
    while s < to {
        let e = s + 3600;
        if e <= first.ts {
            s = e;
            continue;
        }
        let start = value_at(points, s);
        let end = value_at(points, e);
        let lo = points.partition_point(|p| p.ts < s);
        let hi = points.partition_point(|p| p.ts < e);
        let inside = &points[lo..hi];
        let reset = points[lo..points.partition_point(|p| p.ts <= e)]
            .iter()
            .any(|p| p.reset);
        let hour = match (start, end) {
            (Some((a, ga)), Some((b, gb))) => HourDelta {
                hour_start: s,
                delta: (b - a).max(0.0),
                quality: if ga > max_gap || gb > max_gap || reset {
                    Quality::Estimated
                } else {
                    Quality::Ok
                },
            },
            _ => {
                let lo_v = start.map(|v| v.0).or(inside.first().map(|p| p.value));
                let hi_v = end.map(|v| v.0).or(inside.last().map(|p| p.value));
                HourDelta {
                    hour_start: s,
                    delta: match (lo_v, hi_v) {
                        (Some(a), Some(b)) => (b - a).max(0.0),
                        _ => 0.0,
                    },
                    quality: Quality::Gap,
                }
            }
        };
        out.push(hour);
        s = e;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: i64 = 3600;
    const T0: i64 = 1_790_000_000 / H * H;

    fn unwrap(samples: &[(i64, f64)], rollover: Option<f64>) -> Vec<Point> {
        let mut u = CounterUnwrapper::new(rollover);
        samples.iter().filter_map(|&(t, v)| u.push(t, v)).collect()
    }

    /// A counter rising `per_hour` per hour, one sample a minute over `hours`.
    fn ramp(start: f64, per_hour: f64, hours: i64) -> Vec<(i64, f64)> {
        (0..=hours * 60)
            .map(|i| (T0 + i * 60, start + per_hour * i as f64 / 60.0))
            .collect()
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6 * b.abs().max(1.0)
    }

    #[test]
    fn steady_counter_gives_ok_hours() {
        let pts = unwrap(&ramp(1000.0, 120.0, 3), None);
        let hours = hourly_deltas(&pts, T0, T0 + 3 * H, 900);
        assert_eq!(hours.len(), 3);
        for h in &hours {
            assert!(approx(h.delta, 120.0), "{h:?}");
            assert_eq!(h.quality, Quality::Ok);
        }
    }

    #[test]
    fn rollover_adds_the_rollover_value() {
        // A 16-bit counter wrapping at 65536 in the second hour.
        let samples: Vec<(i64, f64)> = ramp(65_000.0, 600.0, 3)
            .into_iter()
            .map(|(t, v)| (t, v % 65_536.0))
            .collect();
        let pts = unwrap(&samples, Some(65_536.0));
        let hours = hourly_deltas(&pts, T0, T0 + 3 * H, 900);
        assert!(hours.iter().all(|h| approx(h.delta, 600.0)), "{hours:?}");
        assert!(hours.iter().all(|h| h.quality == Quality::Ok));
    }

    #[test]
    fn a_drop_without_rollover_is_a_reset_not_negative_energy() {
        let mut samples = ramp(50_000.0, 60.0, 2);
        // Meter cleared at T0 + 90 min: readings continue from zero.
        for s in samples.iter_mut().filter(|s| s.0 >= T0 + 90 * 60) {
            s.1 -= 50_000.0 + 90.0;
        }
        let pts = unwrap(&samples, None);
        let hours = hourly_deltas(&pts, T0, T0 + 2 * H, 900);
        assert!(approx(hours[0].delta, 60.0));
        assert_eq!(hours[0].quality, Quality::Ok);
        // The one-minute step at the reset is lost (1 kWh of 60), and flagged.
        assert!(approx(hours[1].delta, 59.0), "{:?}", hours[1]);
        assert_eq!(hours[1].quality, Quality::Estimated);
        // Same drop with a rollover far above it is still a reset.
        let pts = unwrap(&samples, Some(4_294_967_296.0));
        assert_eq!(
            hourly_deltas(&pts, T0, T0 + 2 * H, 900)[1].quality,
            Quality::Estimated
        );
    }

    #[test]
    fn float_wobble_is_not_a_reset() {
        let pts = unwrap(
            &[(T0, 1000.0), (T0 + 60, 999.9999), (T0 + 120, 1000.5)],
            None,
        );
        assert!(pts.iter().all(|p| !p.reset));
        assert!(approx(pts[2].value, 1000.5));
    }

    #[test]
    fn a_gap_is_shared_over_its_hours_and_marked_estimated() {
        // Samples until T0+1h, then nothing until T0+4h; 100/h throughout.
        let samples: Vec<_> = ramp(0.0, 100.0, 5)
            .into_iter()
            .filter(|(t, _)| *t <= T0 + H || *t >= T0 + 4 * H)
            .collect();
        let pts = unwrap(&samples, None);
        let hours = hourly_deltas(&pts, T0, T0 + 5 * H, 900);
        let q: Vec<_> = hours.iter().map(|h| h.quality).collect();
        assert_eq!(
            q,
            [
                Quality::Ok,
                Quality::Estimated,
                Quality::Estimated,
                Quality::Estimated,
                Quality::Ok
            ]
        );
        let total: f64 = hours.iter().map(|h| h.delta).sum();
        assert!(approx(total, 500.0));
        assert!(approx(hours[2].delta, 100.0));
    }

    #[test]
    fn no_data_after_an_hour_is_a_gap() {
        // Data stops at T0 + 90 min.
        let samples: Vec<_> = ramp(0.0, 60.0, 3)
            .into_iter()
            .filter(|(t, _)| *t <= T0 + 90 * 60)
            .collect();
        let pts = unwrap(&samples, None);
        let hours = hourly_deltas(&pts, T0, T0 + 3 * H, 900);
        assert_eq!(hours[0].quality, Quality::Ok);
        assert_eq!(hours[1].quality, Quality::Gap);
        assert!(approx(hours[1].delta, 30.0));
        assert_eq!(hours[2].quality, Quality::Gap);
        assert_eq!(hours[2].delta, 0.0);
        // Hours before the first sample are not reported at all.
        assert_eq!(hourly_deltas(&pts, T0 - 2 * H, T0 + H, 900).len(), 1);
    }

    #[test]
    fn compacted_points_give_the_same_hours() {
        let pts = unwrap(&ramp(10.0, 37.0, 4), None);
        let full = hourly_deltas(&pts, T0, T0 + 4 * H, 900);
        let mut compacted = Vec::new();
        for h in 0..=4 {
            let chunk: Vec<_> = pts
                .iter()
                .copied()
                .filter(|p| p.ts >= T0 + h * H && p.ts < T0 + (h + 1) * H)
                .collect();
            compacted.extend(compact(chunk));
        }
        assert!(compacted.len() < pts.len());
        assert_eq!(full, hourly_deltas(&compacted, T0, T0 + 4 * H, 900));
    }

    #[test]
    fn kvar_is_integrated_to_kvarh() {
        let mut i = RateIntegrator::new(900);
        let pts: Vec<_> = (0..=120)
            .filter_map(|m| i.push(T0 + m * 60, 300.0))
            .collect();
        let hours = hourly_deltas(&pts, T0, T0 + 2 * H, 900);
        assert!(approx(hours[0].delta, 300.0) && approx(hours[1].delta, 300.0));
    }
}
