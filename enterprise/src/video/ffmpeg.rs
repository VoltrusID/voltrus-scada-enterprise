//! Frames from RTSP cameras through an external `ffmpeg`: it speaks RTSP over
//! TCP with Basic and Digest authentication and decodes H.264, H.265 and MJPEG,
//! which the server then hands out as JPEG snapshots or as an MJPEG live view.
//!
//! `ffmpeg` must be on the PATH, or `VOLTRUS_FFMPEG` must name it.

use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};

/// Longest a snapshot may take (connect, first key frame, encode).
pub const SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(20);
/// Live views open at once, over all cameras; each one is an `ffmpeg` process.
pub const MAX_LIVE_VIEWS: usize = 8;
/// Frames per second of the live view.
pub const LIVE_FPS: u32 = 5;

static LIVE_VIEWS: AtomicUsize = AtomicUsize::new(0);

pub fn ffmpeg_bin() -> String {
    std::env::var("VOLTRUS_FFMPEG")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "ffmpeg".to_string())
}

fn base_command(url: &str) -> Command {
    let mut cmd = Command::new(ffmpeg_bin());
    cmd.args([
        "-nostdin",
        "-hide_banner",
        "-loglevel",
        "error",
        "-rtsp_transport",
        "tcp",
        // Give up on a camera that does not answer (microseconds).
        "-timeout",
        "10000000",
        "-i",
        url,
        "-an",
    ]);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    cmd
}

fn spawn(cmd: &mut Command) -> Result<Child> {
    cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!(
                "ffmpeg was not found: install it on the server (for example `apt install ffmpeg`) or set VOLTRUS_FFMPEG"
            )
        } else {
            anyhow::anyhow!("starting ffmpeg: {e}")
        }
    })
}

/// The camera URL with its password replaced, for logs and messages.
pub fn redact(url: &str) -> String {
    match (url.find("://"), url.find('@')) {
        (Some(scheme), Some(at)) if at > scheme => {
            let creds = &url[scheme + 3..at];
            let user = creds.split(':').next().unwrap_or("");
            format!("{}{user}:***{}", &url[..scheme + 3], &url[at..])
        }
        _ => url.to_string(),
    }
}

/// One JPEG frame from the camera.
pub async fn grab_jpeg(url: &str) -> Result<Vec<u8>> {
    let mut cmd = base_command(url);
    cmd.args([
        "-frames:v",
        "1",
        "-q:v",
        "3",
        "-f",
        "image2",
        "-c:v",
        "mjpeg",
        "pipe:1",
    ]);
    let child = spawn(&mut cmd)?;
    let out = tokio::time::timeout(SNAPSHOT_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "camera {} gave no frame within {}s",
                redact(url),
                SNAPSHOT_TIMEOUT.as_secs()
            )
        })?
        .context("waiting for ffmpeg")?;
    if !out.status.success() || !out.stdout.starts_with(&[0xFF, 0xD8]) {
        let err = String::from_utf8_lossy(&out.stderr);
        let err = err.lines().last().unwrap_or("no frame").trim().to_string();
        bail!(
            "snapshot from {} failed: {}",
            redact(url),
            err.replace(url, &redact(url))
        );
    }
    Ok(out.stdout)
}

/// A slot for one live view; dropping it frees the slot.
pub struct LiveSlot(());

impl Drop for LiveSlot {
    fn drop(&mut self) {
        LIVE_VIEWS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn take_live_slot() -> Option<LiveSlot> {
    let prev = LIVE_VIEWS.fetch_add(1, Ordering::SeqCst);
    if prev >= MAX_LIVE_VIEWS {
        LIVE_VIEWS.fetch_sub(1, Ordering::SeqCst);
        return None;
    }
    Some(LiveSlot(()))
}

/// The `Content-Type` of [`spawn_live`]'s output.
pub const LIVE_CONTENT_TYPE: &str = "multipart/x-mixed-replace;boundary=ffmpeg";

/// Start an MJPEG stream of the camera (`multipart/x-mixed-replace`, which an
/// `<img>` element plays). The process ends when its output is dropped.
pub fn spawn_live(url: &str) -> Result<Child> {
    let mut cmd = base_command(url);
    let fps = LIVE_FPS.to_string();
    cmd.args([
        "-r",
        &fps,
        "-vf",
        "scale='min(1280,iw)':-2",
        "-c:v",
        "mjpeg",
        "-q:v",
        "6",
        "-f",
        "mpjpeg",
        "pipe:1",
    ]);
    // Errors of a live view only matter while debugging; do not fill a pipe.
    cmd.stderr(Stdio::null());
    spawn(&mut cmd)
}

/// The live stream as body chunks, holding the process and the slot until the
/// viewer goes away.
pub fn live_stream(
    mut child: Child,
    slot: LiveSlot,
) -> impl futures_util::Stream<Item = std::io::Result<axum::body::Bytes>> {
    let stdout = child.stdout.take();
    futures_util::stream::unfold((child, stdout, slot), |(child, stdout, slot)| async move {
        let mut stdout = stdout?;
        let mut buf = vec![0u8; 64 * 1024];
        match stdout.read(&mut buf).await {
            Ok(0) => None,
            Ok(n) => {
                buf.truncate(n);
                Some((
                    Ok(axum::body::Bytes::from(buf)),
                    (child, Some(stdout), slot),
                ))
            }
            Err(e) => Some((Err(e), (child, None, slot))),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_the_password() {
        assert_eq!(
            redact("rtsp://admin:s3cret@10.0.0.5:554/Streaming/101"),
            "rtsp://admin:***@10.0.0.5:554/Streaming/101"
        );
        assert_eq!(redact("rtsp://10.0.0.5/live"), "rtsp://10.0.0.5/live");
    }

    #[test]
    fn live_slots_are_bounded() {
        let slots: Vec<_> = std::iter::from_fn(take_live_slot)
            .take(MAX_LIVE_VIEWS + 2)
            .collect();
        assert_eq!(slots.len(), MAX_LIVE_VIEWS);
        assert!(take_live_slot().is_none());
        drop(slots);
        assert!(take_live_slot().is_some());
    }

    #[tokio::test]
    async fn missing_ffmpeg_is_a_clear_error() {
        // SAFETY: tests in this module do not read VOLTRUS_FFMPEG concurrently
        // except this one.
        unsafe { std::env::set_var("VOLTRUS_FFMPEG", "/nonexistent/ffmpeg-for-test") };
        let err = grab_jpeg("rtsp://127.0.0.1:1/x")
            .await
            .unwrap_err()
            .to_string();
        unsafe { std::env::remove_var("VOLTRUS_FFMPEG") };
        assert!(err.contains("ffmpeg was not found"), "{err}");
    }
}
