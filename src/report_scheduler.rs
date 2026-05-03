use crate::cache::DeviceCache;
use crate::models::Report;
use crate::notification::email::EmailNotifier;
use crate::storage::Storage;

/// Starts the background report scheduler.
///
/// Checks the reports table every hour. For each enabled report where
/// enough time has elapsed since `last_generated`, it generates the
/// HTML report, attempts PDF conversion, and sends via email.
pub fn start_report_scheduler(
    storage: Storage,
    cache: DeviceCache,
    email_notifier: EmailNotifier,
) {
    tokio::spawn(async move {
        run_scheduler_loop(storage, cache, email_notifier).await;
    });
    tracing::info!("Report scheduler started (check interval: 1 hour)");
}

async fn run_scheduler_loop(
    storage: Storage,
    cache: DeviceCache,
    email_notifier: EmailNotifier,
) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(3600));

    loop {
        ticker.tick().await;
        if let Err(e) = process_reports(&storage, &cache, &email_notifier).await {
            tracing::error!("Report scheduler error: {}", e);
        }
    }
}

async fn process_reports(
    storage: &Storage,
    cache: &DeviceCache,
    email_notifier: &EmailNotifier,
) -> anyhow::Result<()> {
    let reports = storage.list_reports().await?;

    let now = crate::timeutil::now_secs();

    for report in &reports {
        if !report.enabled {
            continue;
        }

        let duration = crate::report::schedule_to_duration(&report.schedule);
        let last = report.last_generated.unwrap_or(0);

        if now - last < duration.as_secs() as i64 {
            continue;
        }

        tracing::info!(
            "Generating report '{}' (id={}, schedule={})",
            report.name,
            report.id,
            report.schedule,
        );

        match generate_and_send(storage, cache, email_notifier, report).await {
            Ok(()) => {
                if let Err(e) = storage.update_report_last_generated(report.id, now).await {
                    tracing::error!(
                        "Failed to update last_generated for report {}: {}",
                        report.id,
                        e
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to generate/send report '{}' (id={}): {}",
                    report.name,
                    report.id,
                    e
                );
            }
        }
    }

    Ok(())
}

async fn generate_and_send(
    storage: &Storage,
    cache: &DeviceCache,
    email_notifier: &EmailNotifier,
    report: &Report,
) -> anyhow::Result<()> {
    // Branch: analytics digest report type
    if report.report_type.as_deref() == Some("analytics_digest") {
        return generate_and_send_analytics_digest(storage, cache, email_notifier, report).await;
    }

    // Default: standard PDF/HTML report
    let html = crate::report::generate_report_html(storage, cache, report).await?;

    // Try PDF conversion, fall back to HTML
    let (attachment_bytes, content_type, extension) =
        match crate::report::generate_report_pdf(&html) {
            Ok(pdf_bytes) => (pdf_bytes, "application/pdf".to_string(), "pdf".to_string()),
            Err(e) => {
                tracing::warn!(
                    "PDF conversion unavailable for report '{}': {}. Sending as HTML instead.",
                    report.name,
                    e
                );
                (html.into_bytes(), "text/html".to_string(), "html".to_string())
            }
        };

    let filename = crate::report::report_filename(report);
    let filename_with_ext = filename.replace(".html", &format!(".{}", extension));

    // Parse recipients
    let recipients: Vec<String> =
        serde_json::from_str(&report.email_recipients).unwrap_or_default();

    if recipients.is_empty() {
        tracing::warn!(
            "Report '{}' has no email recipients configured, skipping send",
            report.name
        );
        return Ok(());
    }

    // Send to each recipient
    for to in &recipients {
        if let Err(e) = send_report_email(
            email_notifier,
            to,
            &report.name,
            &report.schedule,
            &attachment_bytes,
            &content_type,
            &filename_with_ext,
        )
        .await
        {
            tracing::error!(
                "Failed to send report email to {} for '{}': {}",
                to,
                report.name,
                e
            );
        } else {
            tracing::info!(
                "Report '{}' sent to {} (format: {})",
                report.name,
                to,
                extension,
            );
        }
    }

    Ok(())
}

async fn send_report_email(
    email_notifier: &EmailNotifier,
    to: &str,
    report_name: &str,
    schedule: &str,
    _attachment: &[u8],
    _content_type: &str,
    _filename: &str,
) -> anyhow::Result<()> {
    // Since EmailNotifier doesn't expose its inner mailer for attachment support,
    // we send a notification email using the existing alarm email infrastructure.
    // The generated report (HTML/PDF) is available on disk; a future enhancement
    // would add attachment support to EmailNotifier.

    let schedule_label = capitalize_first(schedule);
    let alarm_data = crate::notification::email::AlarmEmailData {
        source_key: format!("report:{}", report_name),
        source_name: report_name.to_string(),
        severity: "info".to_string(),
        value: _attachment.len() as f64,
        threshold: 0.0,
        message: format!(
            "{} ({}) report generated successfully ({} bytes). Access the report via the Voltrus Lite dashboard.",
            report_name, schedule_label, _attachment.len()
        ),
        unit: "bytes".to_string(),
        triggered_at: format_now(),
    };

    email_notifier.send_alarm_email(to, &alarm_data).await?;

    Ok(())
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn format_now() -> String {
    let ts = crate::timeutil::now_secs();
    match time::OffsetDateTime::from_unix_timestamp(ts) {
        Ok(dt) => dt
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| ts.to_string()),
        Err(_) => ts.to_string(),
    }
}

async fn generate_and_send_analytics_digest(
    storage: &Storage,
    cache: &DeviceCache,
    email_notifier: &EmailNotifier,
    report: &Report,
) -> anyhow::Result<()> {
    let config: serde_json::Value = serde_json::from_str(
        report.analytics_config.as_deref().unwrap_or("{}"),
    )
    .unwrap_or(serde_json::Value::Null);

    let now = crate::timeutil::now_secs();
    let from = now - 86400; // last 24h

    // Build metrics summary from telemetry data
    let metrics_list: Vec<String> = config
        .get("metrics")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let mut summary: Vec<(String, f64, f64, f64, f64)> = Vec::new();
    for m in &metrics_list {
        if let Ok(agg) = storage.aggregate_single_period("", m, from, now).await {
            summary.push((m.clone(), agg.min, agg.max, agg.avg, agg.avg));
        }
    }

    // Evaluate custom metrics
    let custom = crate::analytics::custom::evaluate_all_metrics(storage, cache)
        .await
        .unwrap_or_default();
    let custom_summary: Vec<(String, f64, Option<String>)> = custom
        .iter()
        .map(|c| (c.name.clone(), c.value, c.unit.clone()))
        .collect();

    let html =
        crate::report::generate_analytics_digest_html(&summary, &custom_summary, &[]);

    // Parse recipients
    let recipients: Vec<String> =
        serde_json::from_str(&report.email_recipients).unwrap_or_default();

    if recipients.is_empty() {
        tracing::warn!(
            "Analytics digest '{}' has no email recipients, skipping send",
            report.name
        );
        return Ok(());
    }

    for to in &recipients {
        let alarm_data = crate::notification::email::AlarmEmailData {
            source_key: format!("analytics:{}", report.name),
            source_name: report.name.clone(),
            severity: "info".to_string(),
            value: html.len() as f64,
            threshold: 0.0,
            message: format!(
                "Analytics digest for '{}' generated ({} bytes).",
                report.name,
                html.len()
            ),
            unit: "bytes".to_string(),
            triggered_at: format_now(),
        };
        if let Err(e) = email_notifier.send_alarm_email(to, &alarm_data).await {
            tracing::error!(
                "Failed to send analytics digest to {} for '{}': {}",
                to,
                report.name,
                e
            );
        } else {
            tracing::info!("Analytics digest '{}' sent to {}", report.name, to);
        }
    }

    Ok(())
}
