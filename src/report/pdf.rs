use anyhow::{Context, Result};

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

    let output = child
        .wait_with_output()
        .context("wkhtmltopdf process failed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wkhtmltopdf failed (exit {}): {}", output.status, stderr);
    }

    Ok(output.stdout)
}
