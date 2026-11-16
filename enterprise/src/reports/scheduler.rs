use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use anyhow::Context;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tokio::sync::RwLock;
use voltrus::models::{Report, SmtpConfig, SmtpEncryption};
use voltrus::storage::Storage;

use crate::reports::render::{
    ReportData, collect_report_data, format_timestamp, render_html, render_pdf,
    report_filename_with, schedule_to_duration,
};

/// How often the scheduler looks for reports that are due.
pub const CHECK_INTERVAL_SECS: u64 = 60;

/// The server's live SMTP settings (`Settings › System › SMTP` writes them), so
/// a server configured after start sends reports without a restart.
static LIVE_SMTP: OnceLock<Arc<RwLock<Option<SmtpConfig>>>> = OnceLock::new();

/// Called with the core's SMTP settings handle when the routes are built.
pub fn set_live_smtp(handle: Arc<RwLock<Option<SmtpConfig>>>) {
    let _ = LIVE_SMTP.set(handle);
}

async fn current_smtp(fallback: &Option<SmtpConfig>) -> Option<SmtpConfig> {
    match LIVE_SMTP.get() {
        Some(live) => live.read().await.clone(),
        None => fallback.clone(),
    }
}

/// Where generated reports are kept: `reports/` next to the database.
pub fn reports_dir(db_path: &str) -> PathBuf {
    Path::new(db_path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join("reports")
}

/// Starts the background report scheduler.
///
/// Every [`CHECK_INTERVAL_SECS`] it generates each enabled report whose period
/// (1, 7 or 30 days) has passed since its last run (a new report runs on the
/// next check), saves the PDF and HTML under [`reports_dir`] and emails the PDF
/// to the recipients.
pub fn start_report_scheduler(
    storage: Storage,
    db_path: String,
    smtp_at_start: Option<SmtpConfig>,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(CHECK_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            // Only the active node of a hot standby pair sends reports.
            if !voltrus::node_role::is_active() {
                continue;
            }
            if let Err(e) = process_reports(&storage, &db_path, &smtp_at_start).await {
                tracing::error!("Report scheduler error: {e:#}");
            }
        }
    });
    tracing::info!("Report scheduler started (checks every {CHECK_INTERVAL_SECS} s)");
}

/// Whether `report` is due at `now`.
pub fn is_due(report: &Report, now: i64) -> bool {
    report.enabled
        && now - report.last_generated.unwrap_or(0)
            >= schedule_to_duration(&report.schedule).as_secs() as i64
}

async fn process_reports(
    storage: &Storage,
    db_path: &str,
    smtp_at_start: &Option<SmtpConfig>,
) -> anyhow::Result<()> {
    let now = voltrus::timeutil::now_secs();
    for report in storage.list_reports().await? {
        if !is_due(&report, now) {
            continue;
        }
        tracing::info!(
            "Generating report '{}' (id={}, schedule={})",
            report.name,
            report.id,
            report.schedule
        );
        let outcome = run_report(storage, db_path, smtp_at_start, &report).await;
        // A run counts even when mail failed: the files are saved and the error
        // is logged, and retrying every minute would flood a broken relay.
        if let Err(e) = storage.update_report_last_generated(report.id, now).await {
            tracing::error!(
                "Failed to update last_generated for report {}: {e}",
                report.id
            );
        }
        match outcome {
            Ok(summary) => tracing::info!("Report '{}': {summary}", report.name),
            Err(e) => tracing::error!("Report '{}' (id={}) failed: {e:#}", report.name, report.id),
        }
    }
    Ok(())
}

/// Generate, save and mail one report. Returns a one-line summary.
pub async fn run_report(
    storage: &Storage,
    db_path: &str,
    smtp_at_start: &Option<SmtpConfig>,
    report: &Report,
) -> anyhow::Result<String> {
    let data = collect_report_data(storage, report).await?;
    let pdf = render_pdf(&data);
    let html = render_html(&data);

    let dir = reports_dir(db_path).join(report.id.to_string());
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let pdf_name = report_filename_with(report, "pdf");
    let pdf_path = dir.join(&pdf_name);
    std::fs::write(&pdf_path, &pdf).with_context(|| format!("writing {}", pdf_path.display()))?;
    std::fs::write(dir.join(report_filename_with(report, "html")), &html)?;

    let recipients: Vec<String> =
        serde_json::from_str(&report.email_recipients).unwrap_or_default();
    if recipients.is_empty() {
        return Ok(format!("saved {} (no recipients)", pdf_path.display()));
    }
    let Some(smtp) = current_smtp(smtp_at_start).await else {
        anyhow::bail!(
            "saved {} but SMTP is not configured (Settings › System › SMTP), so nothing was sent",
            pdf_path.display()
        );
    };
    let mut sent = 0;
    let mut failures = Vec::new();
    for to in &recipients {
        match send_report(&smtp, to, &data, &pdf, &pdf_name).await {
            Ok(()) => sent += 1,
            Err(e) => failures.push(format!("{to}: {e:#}")),
        }
    }
    if failures.is_empty() {
        Ok(format!(
            "saved {}, emailed to {sent} recipient(s)",
            pdf_path.display()
        ))
    } else {
        anyhow::bail!(
            "saved {}, emailed to {sent} of {}; failed: {}",
            pdf_path.display(),
            recipients.len(),
            failures.join("; ")
        )
    }
}

fn transport(cfg: &SmtpConfig) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = (!cfg.username.is_empty())
        .then(|| Credentials::new(cfg.username.clone(), cfg.password.clone()));
    let builder = match cfg.encryption {
        SmtpEncryption::StartTls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host)?.port(cfg.port)
        }
        SmtpEncryption::ImplicitTls => {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)?.port(cfg.port)
        }
        SmtpEncryption::None => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.host).port(cfg.port)
        }
    };
    let builder = match creds {
        Some(c) => builder.credentials(c),
        None => builder,
    };
    Ok(builder.build())
}

/// The email for one recipient: a short text and HTML summary, the PDF attached.
pub fn build_report_email(
    from: Mailbox,
    to: Mailbox,
    data: &ReportData,
    pdf: &[u8],
    pdf_name: &str,
) -> anyhow::Result<Message> {
    let period = format!(
        "{} to {} (UTC)",
        format_timestamp(data.from),
        format_timestamp(data.to)
    );
    let text = format!(
        "{name}\n{period}\n\nDevices: {devices}\nMetrics: {metrics}\nAlarms in period: {alarms}\n\nThe full report is attached as a PDF.\n",
        name = data.name,
        devices = data.device_count,
        metrics = data.metrics.len(),
        alarms = data.alarms.len(),
    );
    let html = format!(
        "<p><strong>{name}</strong><br>{period}</p><p>Devices: {devices}<br>Metrics: {metrics}<br>Alarms in period: {alarms}</p><p>The full report is attached as a PDF.</p>",
        name = crate::reports::render::html_escape(&data.name),
        devices = data.device_count,
        metrics = data.metrics.len(),
        alarms = data.alarms.len(),
    );
    let subject = format!(
        "[Voltrus] {} ({} report)",
        data.name,
        crate::reports::render::capitalize_first(&data.schedule)
    );
    Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .multipart(
            MultiPart::mixed()
                .multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::plain(text))
                        .singlepart(SinglePart::html(html)),
                )
                .singlepart(
                    Attachment::new(pdf_name.to_string())
                        .body(pdf.to_vec(), ContentType::parse("application/pdf")?),
                ),
        )
        .context("building the report email")
}

async fn send_report(
    smtp: &SmtpConfig,
    to: &str,
    data: &ReportData,
    pdf: &[u8],
    pdf_name: &str,
) -> anyhow::Result<()> {
    let from: Mailbox = format!("{} <{}>", smtp.from_name, smtp.from_address)
        .parse()
        .with_context(|| format!("invalid from address {}", smtp.from_address))?;
    let to_box: Mailbox = to
        .parse()
        .with_context(|| format!("invalid address {to}"))?;
    let email = build_report_email(from, to_box, data, pdf, pdf_name)?;
    transport(smtp)?
        .send(email)
        .await
        .map_err(|e| anyhow::anyhow!("SMTP: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(schedule: &str, last: Option<i64>, enabled: bool) -> Report {
        Report {
            id: 1,
            name: "Daily".into(),
            schedule: schedule.into(),
            enabled,
            device_ids: "all".into(),
            email_recipients: "[\"ops@example.com\"]".into(),
            last_generated: last,
            created_at: 0,
        }
    }

    #[test]
    fn due_rules() {
        let now = 10 * 86_400;
        assert!(
            is_due(&report("daily", None, true), now),
            "a new report runs at once"
        );
        assert!(!is_due(&report("daily", None, false), now));
        assert!(!is_due(&report("daily", Some(now - 3600), true), now));
        assert!(is_due(&report("daily", Some(now - 86_400), true), now));
        assert!(!is_due(
            &report("weekly", Some(now - 6 * 86_400), true),
            now
        ));
        assert!(is_due(&report("weekly", Some(now - 7 * 86_400), true), now));
    }

    #[test]
    fn reports_dir_is_next_to_the_database() {
        assert_eq!(
            reports_dir("/var/lib/voltrus/voltrus.db"),
            Path::new("/var/lib/voltrus/reports")
        );
        assert_eq!(reports_dir("voltrus.db"), Path::new("./reports"));
    }

    #[test]
    fn email_carries_the_pdf() {
        let data = ReportData {
            name: "Line <1>".into(),
            schedule: "daily".into(),
            from: 0,
            to: 86_400,
            device_count: 2,
            metrics: vec![],
            alarms: vec![],
        };
        let pdf = render_pdf(&data);
        let msg = build_report_email(
            "Voltrus <voltrus@example.com>".parse().unwrap(),
            "ops@example.com".parse().unwrap(),
            &data,
            &pdf,
            "report_x_daily.pdf",
        )
        .unwrap();
        let raw = String::from_utf8_lossy(&msg.formatted()).to_string();
        assert!(raw.contains("Subject: [Voltrus] Line <1> (Daily report)"));
        assert!(raw.contains("Content-Type: application/pdf"));
        assert!(raw.contains("filename=\"report_x_daily.pdf\""));
        assert!(raw.contains("Line &lt;1&gt;"));
        assert!(raw.contains("multipart/mixed"));
    }
}
