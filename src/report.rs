use crate::cache::DeviceCache;
use crate::models::{DataPoint, Report};
use crate::storage::Storage;
use anyhow::{Context, Result};

/// Generates the HTML content for a scheduled report.
///
/// Fetches telemetry data for the specified devices (or all devices) within
/// the report's time window, computes summary statistics, and renders a
/// professional dark-themed HTML report.
pub async fn generate_report_html(
    storage: &Storage,
    _cache: &DeviceCache,
    report: &Report,
) -> Result<String> {
    let duration = schedule_to_duration(&report.schedule);
    let now = crate::timeutil::now_secs();
    let from = now - duration.as_secs() as i64;

    // Resolve device IDs
    let device_ids: Vec<String> = if report.device_ids == "all" {
        storage.get_telemetry_device_ids().await.unwrap_or_default()
    } else {
        serde_json::from_str(&report.device_ids).unwrap_or_default()
    };

    // Build device sections
    let mut device_sections = Vec::new();
    for device_id in &device_ids {
        let metrics = storage.get_telemetry_metrics(device_id).await.unwrap_or_default();
        if metrics.is_empty() {
            continue;
        }

        let mut metric_rows = String::new();
        for metric in &metrics {
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

            let trend_chart = build_ascii_chart(&points);

            metric_rows.push_str(&format!(
                r#"<tr>
                    <td style="padding:8px;border-bottom:1px solid #333;">{device_id}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;">{metric}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;">{min:.2}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;">{max:.2}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;">{avg:.2}</td>
                    <td style="padding:8px;border-bottom:1px solid #333;font-family:monospace;font-size:11px;white-space:pre;">{trend_chart}</td>
                </tr>"#,
                device_id = html_escape(device_id),
                metric = html_escape(metric),
            ));
        }

        if !metric_rows.is_empty() {
            device_sections.push(format!(
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
                    <tbody>{metric_rows}</tbody>
                </table>"#,
                device_id = html_escape(device_id),
            ));
        }
    }

    // Fetch recent alarms
    let alarms = storage
        .list_alarms(None, 50)
        .await
        .unwrap_or_default();
    let filtered_alarms: Vec<_> = alarms
        .iter()
        .filter(|a| a.triggered_at >= from)
        .collect();

    let mut alarm_rows = String::new();
    for alarm in &filtered_alarms {
        let severity_color = match alarm.severity.as_str() {
            "critical" => "#ef4444",
            _ => "#f59e0b",
        };
        let triggered = format_triggered_at(alarm.triggered_at);
        alarm_rows.push_str(&format!(
            r#"<tr>
                <td style="padding:6px;border-bottom:1px solid #333;">
                    <span style="color:{severity_color};font-weight:600;">{severity}</span>
                </td>
                <td style="padding:6px;border-bottom:1px solid #333;">{source_name}</td>
                <td style="padding:6px;border-bottom:1px solid #333;">{message}</td>
                <td style="padding:6px;border-bottom:1px solid #333;">{value}{unit_placeholder}</td>
                <td style="padding:6px;border-bottom:1px solid #333;">{triggered}</td>
            </tr>"#,
            severity = alarm.severity.to_uppercase(),
            severity_color = severity_color,
            source_name = html_escape(&alarm.source_name),
            message = html_escape(&alarm.message),
            value = alarm.value,
            unit_placeholder = "",
            triggered = triggered,
        ));
    }

    let from_dt = format_timestamp(from);
    let to_dt = format_timestamp(now);
    let schedule_label = capitalize_first(&report.schedule);

    Ok(format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
    body {{
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        background: #0f172a;
        color: #e2e8f0;
        margin: 0;
        padding: 24px;
        line-height: 1.5;
    }}
    .container {{
        max-width: 960px;
        margin: 0 auto;
        background: #1e293b;
        border-radius: 12px;
        box-shadow: 0 4px 24px rgba(0,0,0,0.4);
        overflow: hidden;
    }}
    .header {{
        background: linear-gradient(135deg, #3b82f6, #8b5cf6);
        color: #ffffff;
        padding: 28px 32px;
    }}
    .header h1 {{
        margin: 0 0 8px 0;
        font-size: 22px;
    }}
    .header p {{
        margin: 0;
        opacity: 0.85;
        font-size: 14px;
    }}
    .body {{
        padding: 28px 32px;
    }}
    h2 {{
        color: #f1f5f9;
        border-bottom: 2px solid #334155;
        padding-bottom: 8px;
        margin-top: 24px;
        font-size: 18px;
    }}
    h3 {{
        font-size: 15px;
    }}
    .footer {{
        padding: 16px 32px;
        background: #0f172a;
        font-size: 12px;
        color: #64748b;
        text-align: center;
    }}
    .stat-grid {{
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 16px;
        margin-bottom: 24px;
    }}
    .stat-card {{
        background: #0f172a;
        border-radius: 8px;
        padding: 16px;
        text-align: center;
    }}
    .stat-card .label {{
        font-size: 12px;
        color: #94a3b8;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }}
    .stat-card .value {{
        font-size: 24px;
        font-weight: 700;
        color: #60a5fa;
        margin-top: 4px;
    }}
    table {{
        font-size: 13px;
    }}
    th {{
        color: #94a3b8;
        font-weight: 600;
        text-transform: uppercase;
        font-size: 11px;
        letter-spacing: 0.5px;
    }}
    td, th {{
        color: #cbd5e1;
    }}
    .no-data {{
        text-align: center;
        color: #64748b;
        padding: 24px;
        font-style: italic;
    }}
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
            <div class="stat-card">
                <div class="label">Devices</div>
                <div class="value">{device_count}</div>
            </div>
            <div class="stat-card">
                <div class="label">Alarms</div>
                <div class="value">{alarm_count}</div>
            </div>
            <div class="stat-card">
                <div class="label">Schedule</div>
                <div class="value">{schedule_label}</div>
            </div>
        </div>

        <h2>Device Metrics</h2>
        {device_content}

        <h2>Alarm Log</h2>
        {alarm_content}
    </div>
    <div class="footer">
        Voltrus Lite SCADA Monitoring System &middot; Generated {generated_at}
    </div>
</div>
</body>
</html>"#,
        report_name = html_escape(&report.name),
        schedule_label = schedule_label,
        from_dt = from_dt,
        to_dt = to_dt,
        device_count = device_ids.len(),
        alarm_count = filtered_alarms.len(),
        device_content = if device_sections.is_empty() {
            r#"<p class="no-data">No telemetry data available for the reporting period.</p>"#.to_string()
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
        generated_at = format_timestamp(now),
    ))
}

/// Convert HTML to PDF using wkhtmltopdf.
///
/// Returns the PDF bytes on success. If wkhtmltopdf is not available,
/// logs a warning and returns an error.
pub fn generate_report_pdf(html: &str) -> Result<Vec<u8>> {
    let mut child = std::process::Command::new("wkhtmltopdf")
        .arg("--quiet")
        .arg("--enable-local-file-access")
        .arg("-")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn wkhtmltopdf. Is it installed?")?;

    use std::io::Write;
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(html.as_bytes())?;
    }

    let output = child.wait_with_output().context("wkhtmltopdf process failed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wkhtmltopdf failed (exit {}): {}", output.status, stderr);
    }

    Ok(output.stdout)
}

/// Generates a filename for a report PDF/HTML based on the report config.
pub fn report_filename(report: &Report) -> String {
    let now = chrono_or_time_format(crate::timeutil::now_secs());
    format!("report_{}_{}.{}", now, report.schedule, "html")
}

/// Converts a schedule string ("daily", "weekly", "monthly") to a Duration.
pub fn schedule_to_duration(schedule: &str) -> std::time::Duration {
    match schedule {
        "weekly" => std::time::Duration::from_secs(7 * 24 * 3600),
        "monthly" => std::time::Duration::from_secs(30 * 24 * 3600),
        _ => std::time::Duration::from_secs(24 * 3600), // daily
    }
}

// ── Internal helpers ──

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn format_timestamp(ts: i64) -> String {
    // Use time crate (already a dependency)
    match time::OffsetDateTime::from_unix_timestamp(ts) {
        Ok(dt) => dt
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| ts.to_string()),
        Err(_) => ts.to_string(),
    }
}

fn format_triggered_at(ts: i64) -> String {
    format_timestamp(ts)
}

fn chrono_or_time_format(ts: i64) -> String {
    match time::OffsetDateTime::from_unix_timestamp(ts) {
        Ok(dt) => dt
            .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
            .unwrap_or_else(|_| ts.to_string()),
        Err(_) => ts.to_string(),
    }
}

/// Build a simple ASCII sparkline chart from data points.
fn build_ascii_chart(points: &[DataPoint]) -> String {
    if points.len() < 2 {
        return String::new();
    }

    let values: Vec<f64> = points.iter().map(|p| p.value).collect();
    let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max_val - min_val;
    if range == 0.0 {
        return "_".repeat(values.len().min(40));
    }

    // Sample down to at most 40 characters
    let step = (values.len() as f64 / 40.0).ceil() as usize;
    let sampled: Vec<f64> = values
        .iter()
        .step_by(step)
        .cloned()
        .collect();

    let bars = ['_', '.', '-', '~', '+', '=', '#'];
    let bar_count = bars.len();
    let mut chart = String::new();
    for v in &sampled {
        let normalized = (v - min_val) / range;
        let idx = ((normalized * (bar_count - 1) as f64).round() as usize).min(bar_count - 1);
        chart.push(bars[idx]);
    }
    chart
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_to_duration_daily() {
        let d = schedule_to_duration("daily");
        assert_eq!(d.as_secs(), 24 * 3600);
    }

    #[test]
    fn test_schedule_to_duration_weekly() {
        let d = schedule_to_duration("weekly");
        assert_eq!(d.as_secs(), 7 * 24 * 3600);
    }

    #[test]
    fn test_schedule_to_duration_monthly() {
        let d = schedule_to_duration("monthly");
        assert_eq!(d.as_secs(), 30 * 24 * 3600);
    }

    #[test]
    fn test_schedule_to_duration_unknown_defaults_to_daily() {
        let d = schedule_to_duration("hourly");
        assert_eq!(d.as_secs(), 24 * 3600);
    }

    #[test]
    fn test_report_filename() {
        let report = Report {
            id: 1,
            name: "Daily Summary".to_string(),
            schedule: "daily".to_string(),
            enabled: true,
            device_ids: "all".to_string(),
            email_recipients: "[]".to_string(),
            last_generated: None,
            created_at: 1700000000,
        };
        let filename = report_filename(&report);
        assert!(filename.starts_with("report_"));
        assert!(filename.ends_with("_daily.html"));
    }

    #[test]
    fn test_report_filename_weekly() {
        let report = Report {
            id: 2,
            name: "Weekly Report".to_string(),
            schedule: "weekly".to_string(),
            enabled: true,
            device_ids: "[\"dev1\"]".to_string(),
            email_recipients: "[]".to_string(),
            last_generated: None,
            created_at: 1700000000,
        };
        let filename = report_filename(&report);
        assert!(filename.ends_with("_weekly.html"));
    }

    #[test]
    fn test_build_ascii_chart() {
        let points = vec![
            DataPoint { timestamp: 1, value: 10.0 },
            DataPoint { timestamp: 2, value: 20.0 },
            DataPoint { timestamp: 3, value: 15.0 },
            DataPoint { timestamp: 4, value: 30.0 },
            DataPoint { timestamp: 5, value: 25.0 },
        ];
        let chart = build_ascii_chart(&points);
        assert!(!chart.is_empty());
        // Should have some characters
        assert!(chart.len() >= 3);
    }

    #[test]
    fn test_build_ascii_chart_empty() {
        let chart = build_ascii_chart(&[]);
        assert!(chart.is_empty());
    }

    #[test]
    fn test_build_ascii_chart_single_point() {
        let chart = build_ascii_chart(&[DataPoint { timestamp: 1, value: 42.0 }]);
        assert!(chart.is_empty());
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("daily"), "Daily");
        assert_eq!(capitalize_first("weekly"), "Weekly");
        assert_eq!(capitalize_first("monthly"), "Monthly");
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }
}
