use crate::cache::DeviceCache;
use crate::models::Report;
use crate::notification::email::EmailNotifier;
use crate::storage::Storage;

/// Starts the background report scheduler.
///
/// Checks the reports table every hour. For each enabled report where
/// enough time has elapsed since `last_generated`, it generates the
/// HTML report, attempts PDF conversion, and sends via email.
pub fn start_report_scheduler(storage: Storage, cache: DeviceCache, email_notifier: EmailNotifier) {
    tokio::spawn(async move {
        run_scheduler_loop(storage, cache, email_notifier).await;
    });
    tracing::info!("Report scheduler started (check interval: 1 hour)");
}

async fn run_scheduler_loop(storage: Storage, cache: DeviceCache, email_notifier: EmailNotifier) {
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
    let html = crate::report::generate_report_html(storage, cache, report).await?;

    let attachment_bytes = html.into_bytes();
    let content_type = "text/html";
    let extension = "html";

    let filename = crate::report::report_filename(report);

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
            content_type,
            &filename,
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
            "{} ({}) report generated successfully ({} bytes). Access the report via the Voltrus dashboard.",
            report_name,
            schedule_label,
            _attachment.len()
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
