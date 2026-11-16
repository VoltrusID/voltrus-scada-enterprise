use anyhow::Result;
use voltrus::cache::DeviceCache;
use voltrus::models::Report;
use voltrus::storage::Storage;

use super::{
    build_ascii_chart, capitalize_first, format_timestamp, format_triggered_at, html_escape,
    schedule_to_duration,
};

/// Most alarms one report lists (the newest of the period).
const MAX_ALARMS: i64 = 500;
/// Points kept per metric for the trend line.
const TREND_POINTS: usize = 60;

/// Statistics of one metric over the report period.
#[derive(Debug, Clone)]
pub struct MetricRow {
    pub device_id: String,
    pub metric: String,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub samples: usize,
    /// Values sampled down to at most [`TREND_POINTS`], oldest first.
    pub trend: Vec<f64>,
    /// Text sparkline for the HTML report.
    pub ascii_trend: String,
}

/// One alarm raised in the period.
#[derive(Debug, Clone)]
pub struct AlarmRow {
    pub severity: String,
    pub source_name: String,
    pub message: String,
    pub value: f64,
    pub triggered_at: i64,
}

/// Everything a report shows, independent of its format (HTML or PDF).
#[derive(Debug, Clone)]
pub struct ReportData {
    pub name: String,
    pub schedule: String,
    pub from: i64,
    pub to: i64,
    pub device_count: usize,
    pub metrics: Vec<MetricRow>,
    pub alarms: Vec<AlarmRow>,
}

/// Read the report's devices, their history over the period and the period's alarms.
pub async fn collect_report_data(storage: &Storage, report: &Report) -> Result<ReportData> {
    let duration = schedule_to_duration(&report.schedule);
    let now = voltrus::timeutil::now_secs();
    let from = now - duration.as_secs() as i64;

    let device_ids: Vec<String> = if report.device_ids == "all" {
        storage.get_telemetry_device_ids().await.unwrap_or_default()
    } else {
        serde_json::from_str(&report.device_ids).unwrap_or_default()
    };

    let mut metrics = Vec::new();
    for device_id in &device_ids {
        let names = storage
            .get_telemetry_metrics(device_id)
            .await
            .unwrap_or_default();
        for metric in &names {
            let points = storage
                .query_history_multi(device_id, from, now, Some(metric))
                .await
                .unwrap_or_default();
            if points.is_empty() {
                continue;
            }
            let values: Vec<f64> = points.iter().map(|p| p.value).collect();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let avg = values.iter().sum::<f64>() / values.len() as f64;
            let step = values.len().div_ceil(TREND_POINTS).max(1);
            metrics.push(MetricRow {
                device_id: device_id.clone(),
                metric: metric.clone(),
                min,
                max,
                avg,
                samples: values.len(),
                trend: values.iter().step_by(step).cloned().collect(),
                ascii_trend: build_ascii_chart(&points),
            });
        }
    }

    let alarms = storage
        .list_alarms(None, MAX_ALARMS)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|a| a.triggered_at >= from)
        .map(|a| AlarmRow {
            severity: a.severity,
            source_name: a.source_name,
            message: a.message,
            value: a.value,
            triggered_at: a.triggered_at,
        })
        .collect();

    Ok(ReportData {
        name: report.name.clone(),
        schedule: report.schedule.clone(),
        from,
        to: now,
        device_count: device_ids.len(),
        metrics,
        alarms,
    })
}

/// Generates the HTML content for a scheduled report.
pub async fn generate_report_html(
    storage: &Storage,
    _cache: &DeviceCache,
    report: &Report,
) -> Result<String> {
    Ok(render_html(&collect_report_data(storage, report).await?))
}

/// The report as a self-contained HTML page (dark theme, inline styles).
pub fn render_html(data: &ReportData) -> String {
    let mut device_sections = Vec::new();
    let mut current: Option<&str> = None;
    let mut metric_rows = String::new();
    let flush = |device: Option<&str>, rows: &mut String, out: &mut Vec<String>| {
        if let Some(device_id) = device
            && !rows.is_empty()
        {
            out.push(format!(
                r#"<h3 style="color:#60a5fa;margin-top:24px;">Device: {device_id}</h3>
                <table style="width:100%;border-collapse:collapse;margin-bottom:16px;">
                    <thead>
                        <tr style="background:#1e293b;">
                            <th style="padding:8px;text-align:left;border-bottom:2px solid #334155;">Device</th>
                            <th style="padding:8px;text-align:left;border-bottom:2px solid #334155;">Metric</th>
                            <th style="padding:8px;text-align:right;border-bottom:2px solid #334155;">Min</th>
                            <th style="padding:8px;text-align:right;border-bottom:2px solid #334155;">Max</th>
                            <th style="padding:8px;text-align:right;border-bottom:2px solid #334155;">Avg</th>
                            <th style="padding:8px;text-align:left;border-bottom:2px solid #334155;">Trend</th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>"#,
                device_id = html_escape(device_id),
            ));
            rows.clear();
        }
    };
    for m in &data.metrics {
        if current != Some(m.device_id.as_str()) {
            flush(current, &mut metric_rows, &mut device_sections);
            current = Some(m.device_id.as_str());
        }
        metric_rows.push_str(&format!(
            r#"<tr>
                    <td style="padding:8px;border-bottom:1px solid #333;">{device_id}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;">{metric}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;">{min:.2}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;">{max:.2}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;">{avg:.2}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;font-family:monospace;font-size:11px;white-space:pre;">{trend_chart}</td>
                </tr>"#,
            device_id = html_escape(&m.device_id),
            metric = html_escape(&m.metric),
            min = m.min,
            max = m.max,
            avg = m.avg,
            trend_chart = m.ascii_trend,
        ));
    }
    flush(current, &mut metric_rows, &mut device_sections);

    let mut alarm_rows = String::new();
    for alarm in &data.alarms {
        let severity_color = match alarm.severity.as_str() {
            "critical" => "#ef4444",
            _ => "#f59e0b",
        };
        alarm_rows.push_str(&format!(
            r#"<tr>
                <td style="padding:6px;border-bottom:1px solid #333;">
                    <span style="color:{severity_color};font-weight:600;">{severity}</span>
                </td>
                <td style="padding:6px;border-bottom:1px solid #333;">{source_name}</td>
                <td style="padding:6px;border-bottom:1px solid #333;">{message}</td>
                <td style="padding:6px;border-bottom:1px solid #333;">{value}</td>
                <td style="padding:6px;border-bottom:1px solid #333;">{triggered}</td>
            </tr>"#,
            severity = html_escape(&alarm.severity.to_uppercase()),
            source_name = html_escape(&alarm.source_name),
            message = html_escape(&alarm.message),
            value = alarm.value,
            triggered = format_triggered_at(alarm.triggered_at),
        ));
    }

    let from_dt = format_timestamp(data.from);
    let to_dt = format_timestamp(data.to);
    let schedule_label = capitalize_first(&data.schedule);

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{report_name}</title>
<style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f172a; color: #e2e8f0; margin: 0; padding: 24px; line-height: 1.5; }}
    .container {{ max-width: 960px; margin: 0 auto; background: #1e293b; border-radius: 12px; box-shadow: 0 4px 24px rgba(0,0,0,0.4); overflow: hidden; }}
    .header {{ background: linear-gradient(135deg, #3b82f6, #8b5cf6); color: #ffffff; padding: 28px 32px; }}
    .header h1 {{ margin: 0 0 8px 0; font-size: 22px; }}
    .header p {{ margin: 0; opacity: 0.85; font-size: 14px; }}
    .body {{ padding: 28px 32px; }}
    h2 {{ color: #f1f5f9; border-bottom: 2px solid #334155; padding-bottom: 8px; margin-top: 24px; font-size: 18px; }}
    h3 {{ font-size: 15px; }}
    .footer {{ padding: 16px 32px; background: #0f172a; font-size: 12px; color: #64748b; text-align: center; }}
    .stat-grid {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 24px; }}
    .stat-card {{ background: #0f172a; border-radius: 8px; padding: 16px; text-align: center; }}
    .stat-card .label {{ font-size: 12px; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.5px; }}
    .stat-card .value {{ font-size: 24px; font-weight: 700; color: #60a5fa; margin-top: 4px; }}
    table {{ font-size: 13px; }}
    th {{ color: #94a3b8; font-weight: 600; text-transform: uppercase; font-size: 11px; letter-spacing: 0.5px; }}
    td, th {{ color: #cbd5e1; }}
    .no-data {{ text-align: center; color: #64748b; padding: 24px; font-style: italic; }}
</style>
</head>
<body>
<div class="container">
    <div class="header">
        <h1>{report_name}</h1>
        <p>{schedule_label} Report &middot; {from_dt} to {to_dt}</p>
    </div>
    <div class="body">
        <div class="stat-grid">
            <div class="stat-card"><div class="label">Devices</div><div class="value">{device_count}</div></div>
            <div class="stat-card"><div class="label">Alarms</div><div class="value">{alarm_count}</div></div>
            <div class="stat-card"><div class="label">Schedule</div><div class="value">{schedule_label}</div></div>
        </div>

        <h2>Device Metrics</h2>
        {device_content}

        <h2>Alarm Log</h2>
        {alarm_content}
    </div>
    <div class="footer">
        Voltrus SCADA Monitoring System &middot; Generated {generated_at}
    </div>
</div>
</body>
</html>"#,
        report_name = html_escape(&data.name),
        schedule_label = html_escape(&schedule_label),
        device_count = data.device_count,
        alarm_count = data.alarms.len(),
        device_content = if device_sections.is_empty() {
            r#"<p class="no-data">No telemetry data available for the reporting period.</p>"#
                .to_string()
        } else {
            device_sections.join("\n")
        },
        alarm_content = if alarm_rows.is_empty() {
            r#"<p class="no-data">No alarms recorded during the reporting period.</p>"#.to_string()
        } else {
            format!(
                r#"<table style="width:100%;border-collapse:collapse;">
                    <thead>
                        <tr style="background:#1e293b;">
                            <th style="padding:8px;text-align:left;border-bottom:2px solid #334155;">Severity</th>
                            <th style="padding:8px;text-align:left;border-bottom:2px solid #334155;">Source</th>
                            <th style="padding:8px;text-align:left;border-bottom:2px solid #334155;">Message</th>
                            <th style="padding:8px;text-align:right;border-bottom:2px solid #334155;">Value</th>
                            <th style="padding:8px;text-align:left;border-bottom:2px solid #334155;">Time</th>
                        </tr>
                    </thead>
                    <tbody>{alarm_rows}</tbody>
                </table>"#
            )
        },
        generated_at = format_timestamp(data.to),
    )
}
