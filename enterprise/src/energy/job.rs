//! The hourly energy job: counters → `energy_hour`, then the alerts and the
//! daily summary.
//!
//! Counter values come from the historian's full-resolution samples
//! (`Storage::query_history`, raw rows and packed blocks merged), read one hour
//! at a time. `telemetry_hourly` is not used: the core fills it only when the
//! raw tier expires (90 days by default), so it never holds the current month.
//! Each hour is reduced to its first and last sample (plus resets) before the
//! boundary values are interpolated, so a month of 1 s samples is never held
//! in memory.

use std::time::Duration;

use anyhow::Result;
use voltrus::notification::email::EmailNotifier;
use voltrus::storage::Storage;
use voltrus::ws_manager::WsManager;

use super::counter::{self, CounterUnwrapper, Point, Quality, RateIntegrator};
use super::report;
use super::store::{self, HourRow, Meter, MeterRole};
use super::tariff::{Site, format_date, tariff_for};

/// Samples read before and after the range, to find the counter at its edges.
const EDGE_SECS: i64 = 12 * 3600;
/// On every run, hours this far back are computed again (late data).
const RECOMPUTE_SECS: i64 = 24 * 3600;
/// `gap` hours this recent are retried on every run.
const GAP_RETRY_SECS: i64 = 7 * 86_400;
/// The job runs this long after each full hour, so its last samples are stored.
const RUN_DELAY_SECS: i64 = 120;

enum Source<'a> {
    Counter {
        metric: &'a str,
        rollover: Option<f64>,
    },
    Rate {
        metric: &'a str,
    },
}

/// Monotonic points of one tag over `[from, to]`, read hour by hour.
async fn read_points(
    storage: &Storage,
    device_id: &str,
    source: &Source<'_>,
    max_gap: i64,
    from: i64,
    to: i64,
) -> Result<Vec<Point>> {
    let (metric, mut counter, mut rate) = match source {
        Source::Counter { metric, rollover } => {
            (*metric, Some(CounterUnwrapper::new(*rollover)), None)
        }
        Source::Rate { metric } => (*metric, None, Some(RateIntegrator::new(max_gap))),
    };
    let mut out = Vec::new();
    let mut s = from.div_euclid(3600) * 3600;
    while s <= to {
        let samples = storage
            .query_history(device_id, metric, s, (s + 3599).min(to))
            .await?;
        let pts: Vec<Point> = samples
            .into_iter()
            .filter_map(|p| match (&mut counter, &mut rate) {
                (Some(c), _) => c.push(p.timestamp, p.value),
                (_, Some(r)) => r.push(p.timestamp, p.value),
                _ => None,
            })
            .collect();
        out.extend(counter::compact(pts));
        s += 3600;
    }
    Ok(out)
}

/// Compute and store the hours `[from, to)` (hour-aligned) of one meter.
/// Idempotent: the rows of the range are replaced. Returns the rows written.
pub async fn compute_meter(
    storage: &Storage,
    site: &Site,
    meter: &Meter,
    from: i64,
    to: i64,
    now: i64,
) -> Result<usize> {
    let from = from.div_euclid(3600) * 3600;
    let to = to.div_euclid(3600) * 3600;
    if to <= from {
        return Ok(0);
    }
    let read_from = from - EDGE_SECS;
    let read_to = (to + EDGE_SECS).min(now);
    let max_gap = meter.max_gap_secs.max(1);
    let kwh_points = read_points(
        storage,
        &meter.device_id,
        &Source::Counter {
            metric: &meter.kwh_metric,
            rollover: meter.rollover,
        },
        max_gap,
        read_from,
        read_to,
    )
    .await?;
    let kvarh_source = match (&meter.kvarh_metric, &meter.kvar_metric) {
        (Some(m), _) if !m.is_empty() => Some(Source::Counter {
            metric: m,
            rollover: meter.rollover,
        }),
        (_, Some(m)) if !m.is_empty() => Some(Source::Rate { metric: m }),
        _ => None,
    };
    let kvarh_hours = match &kvarh_source {
        Some(src) => {
            let pts =
                read_points(storage, &meter.device_id, src, max_gap, read_from, read_to).await?;
            Some(counter::hourly_deltas(&pts, from, to, max_gap))
        }
        None => None,
    };
    let rows = build_rows(site, meter, &kwh_points, kvarh_hours.as_deref(), from, to);
    let n = rows.len();
    let (meter_id, storage) = (meter.id, storage.clone());
    storage
        .with_conn(move |c| store::replace_hours(c, meter_id, from, to, &rows, now))
        .await?;
    Ok(n)
}

/// Join the kWh and kVArh hours of a meter into rows, with band and multiplier.
pub fn build_rows(
    site: &Site,
    meter: &Meter,
    kwh_points: &[Point],
    kvarh_hours: Option<&[counter::HourDelta]>,
    from: i64,
    to: i64,
) -> Vec<HourRow> {
    let clock = site.clock();
    let max_gap = meter.max_gap_secs.max(1);
    counter::hourly_deltas(kwh_points, from, to, max_gap)
        .into_iter()
        .map(|h| {
            let (kvarh, quality) = match kvarh_hours {
                None => (None, h.quality),
                Some(list) => match list.iter().find(|k| k.hour_start == h.hour_start) {
                    Some(k) => (Some(k.delta * meter.multiplier), h.quality.worst(k.quality)),
                    None => (None, Quality::Gap),
                },
            };
            HourRow {
                meter_id: meter.id,
                hour_start: h.hour_start,
                kwh: h.delta * meter.multiplier,
                kvarh,
                band: clock.band(h.hour_start, &site.wbp_windows),
                quality,
            }
        })
        .collect()
}

/// Recompute a whole local billing month of every meter of a site.
pub async fn recompute_month(
    storage: &Storage,
    site: &Site,
    month: &str,
    now: i64,
) -> Result<usize> {
    let (from, to) = site
        .clock()
        .month_range(month)
        .map_err(anyhow::Error::msg)?;
    let to = to.min(now.div_euclid(3600) * 3600);
    let site_id = site.id;
    let meters = storage
        .with_conn(move |c| store::list_meters(c, Some(site_id)))
        .await?;
    let mut n = 0;
    for m in meters.iter().filter(|m| m.enabled) {
        n += compute_meter(storage, site, m, from, to, now).await?;
    }
    Ok(n)
}

/// Where the next run starts for a meter: a day before its last hour (late
/// data), the oldest recent `gap` hour, or, for a new meter, the start of the
/// previous billing month.
async fn start_for(storage: &Storage, site: &Site, meter: &Meter, now: i64) -> Result<i64> {
    let id = meter.id;
    let (last, gap) = storage
        .with_conn(move |c| {
            Ok((
                store::last_hour(c, id)?,
                store::first_gap_since(c, id, now - GAP_RETRY_SECS)?,
            ))
        })
        .await?;
    let clock = site.clock();
    let from = match last {
        Some(l) => l - RECOMPUTE_SECS,
        None => {
            let (this_month, _) = clock
                .month_range(&clock.month_of(now))
                .map_err(anyhow::Error::msg)?;
            let prev = clock.month_of(this_month - 1);
            clock.month_range(&prev).map_err(anyhow::Error::msg)?.0
        }
    };
    Ok(gap.map_or(from, |g| g.min(from)))
}

/// One run: every enabled meter up to the last full hour, then alerts and the
/// daily summary per site.
pub async fn run_cycle(
    storage: &Storage,
    ws: Option<&WsManager>,
    email: Option<&EmailNotifier>,
    now: i64,
) -> Result<()> {
    let (sites, meters) = storage
        .with_conn(|c| Ok((store::list_sites(c)?, store::list_meters(c, None)?)))
        .await?;
    let end = now.div_euclid(3600) * 3600;
    for meter in meters.iter().filter(|m| m.enabled) {
        let Some(site) = sites.iter().find(|s| s.id == meter.site_id) else {
            continue;
        };
        let from = start_for(storage, site, meter, now).await?;
        if let Err(e) = compute_meter(storage, site, meter, from, end, now).await {
            tracing::warn!(
                "Energy: meter {} ({}) not computed: {e:#}",
                meter.id,
                meter.name
            );
        }
    }
    for site in &sites {
        let incomers: Vec<i64> = meters
            .iter()
            .filter(|m| m.site_id == site.id && m.enabled && m.role == MeterRole::PlnIncomer)
            .map(|m| m.id)
            .collect();
        if incomers.is_empty() {
            continue;
        }
        if let Err(e) = alerts(storage, ws, email, site, &incomers, now).await {
            tracing::warn!("Energy: alerts of site {} failed: {e:#}", site.id);
        }
        if let Err(e) = daily_summary(storage, email, site, &incomers, now).await {
            tracing::warn!("Energy: daily summary of site {} failed: {e:#}", site.id);
        }
    }
    Ok(())
}

/// Start the hourly job. It runs on the active node only and while the
/// licence covers the feature.
pub fn spawn() {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        loop {
            if let Some(rt) = crate::try_rt()
                && voltrus::node_role::is_active()
                && super::licensed(rt.tier())
            {
                let now = voltrus::timeutil::now_secs();
                if let Err(e) = run_cycle(
                    &rt.storage,
                    Some(rt.ws_manager.as_ref()),
                    rt.email_notifier.as_ref(),
                    now,
                )
                .await
                {
                    tracing::warn!("Energy job failed: {e:#}");
                }
            }
            let now = voltrus::timeutil::now_secs();
            let next = (now.div_euclid(3600) + 1) * 3600 + RUN_DELAY_SECS;
            tokio::time::sleep(Duration::from_secs((next - now).max(60) as u64)).await;
        }
    });
}

// ── Alerts (spec 3.2) ──

pub fn pf_key(site_id: i64) -> String {
    format!("energy:site{site_id}:pf")
}

pub fn wbp_kw_key(site_id: i64) -> String {
    format!("energy:site{site_id}:wbp_kw")
}

pub fn daily_key(site_id: i64) -> String {
    format!("energy:site{site_id}:daily")
}

async fn alerts(
    storage: &Storage,
    ws: Option<&WsManager>,
    email: Option<&EmailNotifier>,
    site: &Site,
    incomers: &[i64],
    now: i64,
) -> Result<()> {
    let end = now.div_euclid(3600) * 3600;
    let window = site.alerts.pf_window_hours.max(1) as i64 * 3600;
    let ids = incomers.to_vec();
    let rows = storage
        .with_conn(move |c| store::hours_for_meters(c, &ids, end - window, end))
        .await?;

    // Rolling power factor, an early warning before PLN's limit.
    let pf_key = pf_key(site.id);
    if let Some(pf) = report::rolling_pf(&rows) {
        if pf < site.alerts.pf_warn {
            let msg = format!(
                "{}: power factor {:.3} over the last {} h is below {:.2} (PLN bills kVArh below 0.85)",
                site.name, pf, site.alerts.pf_window_hours, site.alerts.pf_warn
            );
            raise(
                storage,
                ws,
                email,
                &pf_key,
                pf,
                site.alerts.pf_warn,
                &msg,
                "low",
                0.005,
            )
            .await?;
        } else {
            resolve(storage, ws, &pf_key, Some(pf)).await?;
        }
    }

    // Load in the last WBP hour against the target (kWh in one hour = average kW).
    let kw_key = wbp_kw_key(site.id);
    let last: Vec<&HourRow> = rows.iter().filter(|r| r.hour_start == end - 3600).collect();
    match (site.alerts.wbp_kw_target, last.first()) {
        (Some(target), Some(first)) if first.band == super::tariff::Band::Wbp => {
            let kw: f64 = last.iter().map(|r| r.kwh).sum();
            if kw > target {
                let msg = format!(
                    "{}: {:.0} kW average in the WBP hour from {}:00, above the {:.0} kW target",
                    site.name,
                    kw,
                    site.clock().hour_of_day(first.hour_start),
                    target
                );
                raise(storage, ws, email, &kw_key, kw, target, &msg, "high", 0.0).await?;
            } else {
                resolve(storage, ws, &kw_key, Some(kw)).await?;
            }
        }
        // Outside WBP (or no target) the peak alarm clears.
        _ => resolve(storage, ws, &kw_key, None).await?,
    }
    Ok(())
}

/// Raise an alarm through the core alarm table, the WebSocket and the
/// notification rules of `key`, unless one is already open.
async fn raise(
    storage: &Storage,
    ws: Option<&WsManager>,
    email: Option<&EmailNotifier>,
    key: &str,
    value: f64,
    threshold: f64,
    message: &str,
    alarm_type: &str,
    deadband: f64,
) -> Result<()> {
    if storage.has_any_unresolved_alarm(key).await? {
        return Ok(());
    }
    let id = storage
        .create_alarm_with_type(
            key, "warning", value, threshold, message, deadband, alarm_type,
        )
        .await?;
    let triggered_at = voltrus::timeutil::now_secs();
    if let Some(ws) = ws {
        ws.broadcast_alarm(voltrus::ws_types::ServerMessage::Alarm {
            alarm_id: id,
            tag_id: key.to_string(),
            severity: "warning".into(),
            message: message.to_string(),
            value,
            ts: triggered_at,
        });
    }
    let alarm = voltrus::models::Alarm {
        id,
        source_key: key.to_string(),
        source_name: message.split(':').next().unwrap_or(key).to_string(),
        severity: "warning".into(),
        priority: "warning".into(),
        value,
        threshold,
        message: message.to_string(),
        triggered_at,
        resolved_at: None,
        acknowledged_at: None,
        acknowledged_by: None,
        cooldown_until: None,
        shelved_until: None,
        shelved_by: None,
        shelved_reason: None,
        deadband,
        state: "active".into(),
        alarm_type: alarm_type.to_string(),
    };
    voltrus::notification::dispatch_alarm_notifications(storage.clone(), &alarm, email.cloned())
        .await;
    Ok(())
}

async fn resolve(
    storage: &Storage,
    ws: Option<&WsManager>,
    key: &str,
    value: Option<f64>,
) -> Result<()> {
    let changes = storage.resolve_active_alarms(key, value).await?;
    if let Some(ws) = ws
        && !changes.is_empty()
    {
        ws.broadcast_alarm_updates("resolved", None, &changes);
    }
    Ok(())
}

// ── Daily summary ──

/// `1234567.8` → `Rp 1.234.568` (id) or `Rp 1,234,568` (en).
pub fn rupiah(v: f64, lang: &str) -> String {
    let n = v.round() as i64;
    let digits = n.unsigned_abs().to_string();
    let sep = if lang == "en" { ',' } else { '.' };
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(sep);
        }
        out.push(ch);
    }
    format!("Rp {}{}", if n < 0 { "-" } else { "" }, out)
}

/// The summary line for one day.
pub fn summary_line(site: &Site, date: &str, day: &report::DayRow) -> String {
    let lang = site.alerts.lang.as_str();
    let kvarh = day.kvarh_charge.map(|v| rupiah(v, lang));
    let pf = day
        .cos_phi
        .map(|p| {
            let s = format!("{p:.2}");
            if lang == "en" { s } else { s.replace('.', ",") }
        })
        .unwrap_or_else(|| "-".into());
    if lang == "en" {
        format!(
            "{}, yesterday ({date}): kVArh charge {}, WBP premium {}, cos φ {pf}. Estimate without demand charge, PPJ and VAT.",
            site.name,
            kvarh.unwrap_or_else(|| "n/a (no kVArh meter)".into()),
            rupiah(day.wbp_premium, lang),
        )
    } else {
        format!(
            "{}, kemarin ({date}): biaya kelebihan kVArh {}, premi WBP {}, cos φ {pf}. Perkiraan, belum termasuk biaya beban, PPJ dan PPN.",
            site.name,
            kvarh.unwrap_or_else(|| "- (tanpa meter kVArh)".into()),
            rupiah(day.wbp_premium, lang),
        )
    }
}

/// Once a day, send yesterday's kVArh charge and WBP premium to the
/// notification rules of `energy:site<id>:daily` (email, webhook, Telegram,
/// WhatsApp). Nothing happens while the site has no such rule.
async fn daily_summary(
    storage: &Storage,
    email: Option<&EmailNotifier>,
    site: &Site,
    incomers: &[i64],
    now: i64,
) -> Result<()> {
    if !site.alerts.daily_summary {
        return Ok(());
    }
    let clock = site.clock();
    let today = clock.date(now);
    let Some(yesterday) = today.previous_day() else {
        return Ok(());
    };
    let day = format_date(yesterday);
    let key = daily_key(site.id);
    let settings = storage.get_notification_settings_by_source(&key).await?;
    if settings.iter().all(|s| !s.enabled) {
        return Ok(());
    }
    let (site_id, d) = (site.id, day.clone());
    if storage
        .with_conn(move |c| store::notice_sent(c, site_id, "daily", &d))
        .await?
    {
        return Ok(());
    }
    let (from, to) = (clock.midnight(yesterday), clock.midnight(today));
    let month = clock.month_of(from);
    let ids = incomers.to_vec();
    let (rows, tariffs) = storage
        .with_conn(move |c| {
            Ok((
                store::hours_for_meters(c, &ids, from, to)?,
                store::list_tariffs(c, site_id)?,
            ))
        })
        .await?;
    if rows.is_empty() || tariff_for(&tariffs, &month).is_none() {
        return Ok(());
    }
    let days = report::daily(site, &tariffs, &month, &rows);
    let Some(row) = days.iter().find(|r| r.date == day) else {
        return Ok(());
    };
    let message = summary_line(site, &day, row);
    let delivery = voltrus::notification::Delivery {
        severity: "info".into(),
        source_key: key.clone(),
        source_name: site.name.clone(),
        value: row.kvarh_charge.unwrap_or(0.0),
        threshold: 0.0,
        unit: "IDR".into(),
        message,
        triggered_at: voltrus::notification::webhook::format_triggered_at(now),
        device_id: format!("energy-site-{}", site.id),
    };
    // Marked first: a failing channel must not resend the summary every hour.
    let d = day.clone();
    storage
        .with_conn(move |c| store::mark_notice(c, site_id, "daily", &d, now))
        .await?;
    for err in voltrus::notification::deliver_to_settings(&settings, &delivery, email).await {
        tracing::warn!("Energy daily summary of site {}: {err}", site.id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rupiah_uses_the_language_separator() {
        assert_eq!(rupiah(86_949_720.0, "id"), "Rp 86.949.720");
        assert_eq!(rupiah(86_949_720.4, "en"), "Rp 86,949,720");
        assert_eq!(rupiah(999.0, "id"), "Rp 999");
        assert_eq!(rupiah(0.0, "id"), "Rp 0");
    }
}
