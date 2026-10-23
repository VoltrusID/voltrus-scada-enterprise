use voltrus::models::{DataPoint, Report};

/// Converts a schedule string ("daily", "weekly", "monthly") to a Duration.
pub fn schedule_to_duration(schedule: &str) -> std::time::Duration {
    match schedule {
        "weekly" => std::time::Duration::from_secs(7 * 24 * 3600),
        "monthly" => std::time::Duration::from_secs(30 * 24 * 3600),
        _ => std::time::Duration::from_secs(24 * 3600), // daily
    }
}

/// Generates a filename for a report PDF/HTML based on the report config.
pub fn report_filename(report: &Report) -> String {
    let now = chrono_or_time_format(voltrus::timeutil::now_secs());
    format!("report_{}_{}.{}", now, report.schedule, "html")
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(crate) fn format_timestamp(ts: i64) -> String {
    // Use time crate (already a dependency)
    match time::OffsetDateTime::from_unix_timestamp(ts) {
        Ok(dt) => dt
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| ts.to_string()),
        Err(_) => ts.to_string(),
    }
}

pub(crate) fn format_triggered_at(ts: i64) -> String {
    format_timestamp(ts)
}

pub(crate) fn chrono_or_time_format(ts: i64) -> String {
    match time::OffsetDateTime::from_unix_timestamp(ts) {
        Ok(dt) => dt
            .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
            .unwrap_or_else(|_| ts.to_string()),
        Err(_) => ts.to_string(),
    }
}

/// Build a simple ASCII sparkline chart from data points.
pub(crate) fn build_ascii_chart(points: &[DataPoint]) -> String {
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
    let sampled: Vec<f64> = values.iter().step_by(step).cloned().collect();

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
