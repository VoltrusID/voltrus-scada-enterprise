use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::{RtspResponse, SdpInfo, parse_rtsp_url};

// ── RTSP Client ──

pub struct RtspClient {
    stream: Option<TcpStream>,
    url: String,
    cseq: u32,
    pub(crate) session_id: Option<String>,
    host: String,
    port: u16,
    path: String,
    credentials: Option<(String, String)>,
}

impl RtspClient {
    pub fn new(url: &str) -> Self {
        let (host, port, path, credentials) = parse_rtsp_url(url).unwrap_or_default();
        RtspClient {
            stream: None,
            url: url.to_string(),
            cseq: 1,
            session_id: None,
            host,
            port,
            path,
            credentials,
        }
    }

    /// Connect to the RTSP server via TCP.
    pub async fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect(&addr)
            .await
            .with_context(|| format!("Failed to connect to RTSP server at {}", addr))?;
        self.stream = Some(stream);
        Ok(())
    }

    fn next_cseq(&mut self) -> u32 {
        let cseq = self.cseq;
        self.cseq += 1;
        cseq
    }

    fn auth_header(&self) -> String {
        if let Some((user, pass)) = &self.credentials {
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                format!("{}:{}", user, pass),
            );
            format!("Authorization: Basic {}\r\n", encoded)
        } else {
            String::new()
        }
    }

    /// Build a complete RTSP request string.
    pub fn build_request(&mut self, method: &str, extra_headers: &str) -> String {
        let cseq = self.next_cseq();
        let mut req = format!("{} {} RTSP/1.0\r\nCSeq: {}\r\n", method, self.url, cseq);
        req.push_str(&self.auth_header());
        if let Some(ref sid) = self.session_id {
            req.push_str(&format!("Session: {}\r\n", sid));
        }
        if !extra_headers.is_empty() {
            req.push_str(extra_headers);
        }
        req.push_str("\r\n");
        req
    }

    /// Read an RTSP response from the stream.
    async fn read_response(&mut self) -> Result<RtspResponse> {
        let stream = self
            .stream
            .as_mut()
            .context("Not connected to RTSP server")?;

        let mut buf = vec![0u8; 8192];
        let mut total = 0;

        // Read until we get a complete response (headers + empty line + body)
        loop {
            let n = stream
                .read(&mut buf[total..])
                .await
                .context("Failed to read RTSP response")?;
            if n == 0 {
                anyhow::bail!("Connection closed by RTSP server");
            }
            total += n;

            let text = String::from_utf8_lossy(&buf[..total]);
            // Check for end of headers
            if let Some(header_end) = text.find("\r\n\r\n") {
                let content_length: usize = text
                    .lines()
                    .find(|l| l.to_lowercase().starts_with("content-length:"))
                    .and_then(|l| l.split(':').nth(1))
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(0);
                let body_start = header_end + 4;
                if total >= body_start + content_length {
                    break;
                }
            }

            if total >= buf.len() {
                buf.resize(buf.len() * 2, 0);
            }
        }

        let text = String::from_utf8_lossy(&buf[..total]).to_string();
        RtspResponse::parse(&text)
    }

    /// Send RTSP OPTIONS command.
    pub async fn send_options(&mut self) -> Result<RtspResponse> {
        let req = self.build_request("OPTIONS", "");
        let stream = self
            .stream
            .as_mut()
            .context("Not connected to RTSP server")?;
        stream
            .write_all(req.as_bytes())
            .await
            .context("Failed to send OPTIONS")?;
        stream.flush().await?;
        self.read_response().await
    }

    /// Send RTSP DESCRIBE command and parse SDP.
    pub async fn send_describe(&mut self) -> Result<SdpInfo> {
        let extra = "Accept: application/sdp\r\n";
        let req = self.build_request("DESCRIBE", extra);
        let stream = self
            .stream
            .as_mut()
            .context("Not connected to RTSP server")?;
        stream
            .write_all(req.as_bytes())
            .await
            .context("Failed to send DESCRIBE")?;
        stream.flush().await?;
        let resp = self.read_response().await?;
        if resp.status_code != 200 {
            anyhow::bail!("DESCRIBE failed: {} {}", resp.status_code, resp.status_text);
        }
        SdpInfo::parse(&resp.body)
    }

    /// Send RTSP SETUP command. Returns the session ID.
    pub async fn send_setup(&mut self, track_url: &str) -> Result<String> {
        let transport = "Transport: RTP/AVP;unicast;client_port=50000-50001\r\n";
        // Temporarily use track_url as the request URL
        let saved_url = self.url.clone();
        self.url = track_url.to_string();
        let req = self.build_request("SETUP", transport);
        self.url = saved_url;

        let stream = self
            .stream
            .as_mut()
            .context("Not connected to RTSP server")?;
        stream
            .write_all(req.as_bytes())
            .await
            .context("Failed to send SETUP")?;
        stream.flush().await?;
        let resp = self.read_response().await?;
        if resp.status_code != 200 {
            anyhow::bail!("SETUP failed: {} {}", resp.status_code, resp.status_text);
        }
        let session_id = resp
            .header("Session")
            .map(|s| {
                // Session header may have timeout param: "123456;timeout=60"
                s.split(';').next().unwrap_or(s).trim().to_string()
            })
            .context("No Session header in SETUP response")?;
        self.session_id = Some(session_id.clone());
        Ok(session_id)
    }

    /// Send RTSP PLAY command.
    pub async fn send_play(&mut self) -> Result<()> {
        let req = self.build_request("PLAY", "");
        let stream = self
            .stream
            .as_mut()
            .context("Not connected to RTSP server")?;
        stream
            .write_all(req.as_bytes())
            .await
            .context("Failed to send PLAY")?;
        stream.flush().await?;
        let resp = self.read_response().await?;
        if resp.status_code != 200 {
            anyhow::bail!("PLAY failed: {} {}", resp.status_code, resp.status_text);
        }
        Ok(())
    }

    /// Send RTSP TEARDOWN command.
    pub async fn send_teardown(&mut self) -> Result<()> {
        if self.stream.is_none() {
            return Ok(());
        }
        let req = self.build_request("TEARDOWN", "");
        let stream = self
            .stream
            .as_mut()
            .context("Not connected to RTSP server")?;
        // Best-effort teardown; ignore errors if the connection is already closed
        let _ = stream.write_all(req.as_bytes()).await;
        let _ = stream.flush().await;
        Ok(())
    }

    /// Capture a snapshot from the RTSP stream.
    ///
    /// Connects, sends DESCRIBE to determine codec, SETUP+PLAY to start streaming,
    /// then reads data looking for JPEG markers (for MJPEG streams) or returns
    /// raw data (for H.264 streams). For non-MJPEG streams, returns a placeholder
    /// indicating transcoding is required.
    pub async fn capture_snapshot(&mut self) -> Result<Vec<u8>> {
        self.connect().await?;
        self.send_options().await?;

        let sdp = self.send_describe().await?;

        // Determine the track URL from SDP
        let track_url = format!("{}trackID=0", self.url.trim_end_matches('/'));
        self.send_setup(&track_url).await?;
        self.send_play().await?;

        let stream = self
            .stream
            .as_mut()
            .context("Not connected to RTSP server")?;

        let mut data = vec![0u8; 65536];
        let total = stream
            .read(&mut data)
            .await
            .context("Failed to read stream data")?;

        let _ = self.send_teardown().await;

        if total == 0 {
            anyhow::bail!("No data received from RTSP stream");
        }

        data.truncate(total);

        if sdp.is_mjpeg() {
            // Try to find JPEG SOI/EOI markers in the RTP payload
            find_jpeg_in_data(&data)
        } else {
            // For H.264 or other codecs, return raw data with a note
            // In production, you would use an external transcoding tool (e.g., ffmpeg)
            Ok(data)
        }
    }
}

impl Default for RtspClient {
    fn default() -> Self {
        RtspClient {
            stream: None,
            url: String::new(),
            cseq: 1,
            session_id: None,
            host: String::new(),
            port: 554,
            path: String::new(),
            credentials: None,
        }
    }
}

/// Search for JPEG SOI (0xFFD8) and EOI (0xFFD9) markers in raw data.
pub(crate) fn find_jpeg_in_data(data: &[u8]) -> Result<Vec<u8>> {
    // Look for JPEG SOI marker
    for i in 0..data.len().saturating_sub(1) {
        if data[i] == 0xFF && data[i + 1] == 0xD8 {
            // Found SOI, now find EOI
            for j in (i + 2)..data.len().saturating_sub(1) {
                if data[j] == 0xFF && data[j + 1] == 0xD9 {
                    return Ok(data[i..=j + 1].to_vec());
                }
            }
            // SOI found but no EOI — return everything from SOI onwards
            return Ok(data[i..].to_vec());
        }
    }
    // No JPEG markers found; return the raw data as-is
    Ok(data.to_vec())
}
