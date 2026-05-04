use std::sync::Arc;

use anyhow::{Context, Result};
use dashmap::DashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::models::{Camera, CameraSnapshot};

// ── RTSP URL Parsing ──

/// Parse an RTSP URL into (host, port, path, optional_credentials).
/// Returns `((host, port, path), Option<(username, password)>)`.
pub fn parse_rtsp_url(
    url: &str,
) -> Result<(String, u16, String, Option<(String, String)>)> {
    if !url.starts_with("rtsp://") {
        anyhow::bail!("URL must start with rtsp://");
    }
    let rest = &url[7..]; // strip "rtsp://"

    // Split credentials from host
    let (credentials, host_part) = if let Some(at_pos) = rest.find('@') {
        let cred = &rest[..at_pos];
        let host = &rest[at_pos + 1..];
        let mut parts = cred.splitn(2, ':');
        let user = parts.next().unwrap_or("").to_string();
        let pass = parts.next().unwrap_or("").to_string();
        (Some((user, pass)), host)
    } else {
        (None, rest)
    };

    // Split host from path
    let (host_port, path) = if let Some(slash_pos) = host_part.find('/') {
        (&host_part[..slash_pos], host_part[slash_pos..].to_string())
    } else {
        (host_part, "/".to_string())
    };

    // Split host and port
    let (host, port) = if host_port.starts_with('[') {
        // IPv6
        let close = host_port
            .find(']')
            .context("Invalid IPv6 address in RTSP URL")?;
        let host = host_port[..=close].to_string();
        let rest = &host_port[close + 1..];
        let port = if rest.starts_with(':') {
            rest[1..].parse::<u16>().unwrap_or(554)
        } else {
            554
        };
        (host, port)
    } else if let Some(colon_pos) = host_port.rfind(':') {
        (
            host_port[..colon_pos].to_string(),
            host_port[colon_pos + 1..]
                .parse::<u16>()
                .context("Invalid port in RTSP URL")?,
        )
    } else {
        (host_port.to_string(), 554)
    };

    if host.is_empty() {
        anyhow::bail!("RTSP URL missing host");
    }

    Ok((host, port, path, credentials))
}

// ── RTSP Response ──

#[derive(Debug, Clone)]
pub struct RtspResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl RtspResponse {
    /// Get a header value by name (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Parse an RTSP response from raw text.
    pub fn parse(raw: &str) -> Result<Self> {
        let mut lines = raw.lines();
        let status_line = lines
            .next()
            .context("Empty RTSP response")?;

        let mut parts = status_line.splitn(3, ' ');
        let _version = parts.next().context("Missing RTSP version")?;
        let status_code: u16 = parts
            .next()
            .context("Missing status code")?
            .parse()
            .context("Invalid status code")?;
        let status_text = parts.next().unwrap_or("OK").to_string();

        let mut headers = Vec::new();
        let mut body_start = false;
        let mut body_lines = Vec::new();

        for line in lines {
            if line.is_empty() {
                body_start = true;
                continue;
            }
            if body_start {
                body_lines.push(line);
            } else if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.push((key, value));
            }
        }

        Ok(RtspResponse {
            status_code,
            status_text,
            headers,
            body: body_lines.join("\r\n"),
        })
    }
}

// ── SDP Info ──

#[derive(Debug, Clone)]
pub struct SdpInfo {
    pub media_type: String,
    pub codec: String,
    pub payload_type: u32,
    pub bandwidth: Option<u32>,
}

impl SdpInfo {
    /// Parse basic SDP content to extract media information.
    pub fn parse(sdp: &str) -> Result<Self> {
        let mut media_type = "video".to_string();
        let mut codec = "unknown".to_string();
        let mut payload_type = 96u32;
        let mut bandwidth: Option<u32> = None;

        for line in sdp.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("m=") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if !parts.is_empty() {
                    media_type = parts[0].to_string();
                }
                if parts.len() >= 4 {
                    payload_type = parts[3].parse().unwrap_or(96);
                }
            } else if let Some(rest) = line.strip_prefix("b=") {
                let parts: Vec<&str> = rest.split(':').collect();
                if parts.len() >= 2 {
                    bandwidth = parts[1].trim().parse().ok();
                }
            } else if line.starts_with("a=rtpmap:") {
                // e.g. a=rtpmap:96 JPEG/90000
                let rest = &line["a=rtpmap:".len()..];
                if let Some(space_pos) = rest.find(' ') {
                    codec = rest[space_pos + 1..]
                        .split('/')
                        .next()
                        .unwrap_or("unknown")
                        .to_string();
                }
            }
        }

        Ok(SdpInfo {
            media_type,
            codec,
            payload_type,
            bandwidth,
        })
    }

    /// Check if this stream uses MJPEG codec (JPEG frames in RTP).
    pub fn is_mjpeg(&self) -> bool {
        self.codec.eq_ignore_ascii_case("jpeg")
    }
}

// ── RTSP Client ──

pub struct RtspClient {
    stream: Option<TcpStream>,
    url: String,
    cseq: u32,
    session_id: Option<String>,
    host: String,
    port: u16,
    path: String,
    credentials: Option<(String, String)>,
}

impl RtspClient {
    pub fn new(url: &str) -> Self {
        let (host, port, path, credentials) =
            parse_rtsp_url(url).unwrap_or_default();
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
    fn build_request(&mut self, method: &str, extra_headers: &str) -> String {
        let cseq = self.next_cseq();
        let mut req = format!(
            "{} {} RTSP/1.0\r\nCSeq: {}\r\n",
            method, self.url, cseq
        );
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
            anyhow::bail!(
                "DESCRIBE failed: {} {}",
                resp.status_code,
                resp.status_text
            );
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
            anyhow::bail!(
                "SETUP failed: {} {}",
                resp.status_code,
                resp.status_text
            );
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
fn find_jpeg_in_data(data: &[u8]) -> Result<Vec<u8>> {
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

// ── Stream Proxy ──

struct CameraState {
    url: String,
    last_snapshot: Option<CameraSnapshot>,
}

pub struct StreamProxy {
    cameras: Arc<DashMap<i64, CameraState>>,
    snapshot_dir: String,
}

impl StreamProxy {
    pub fn new(snapshot_dir: &str) -> Self {
        // Ensure the snapshot directory exists
        let _ = std::fs::create_dir_all(snapshot_dir);
        StreamProxy {
            cameras: Arc::new(DashMap::new()),
            snapshot_dir: snapshot_dir.to_string(),
        }
    }

    pub async fn add_camera(&self, camera: &Camera) {
        self.cameras.insert(
            camera.id,
            CameraState {
                url: camera.url.clone(),
                last_snapshot: None,
            },
        );
    }

    pub async fn remove_camera(&self, camera_id: i64) {
        self.cameras.remove(&camera_id);
    }

    /// Capture a snapshot from the specified camera.
    pub async fn capture_snapshot(
        &self,
        camera: &Camera,
        alarm_source: Option<&str>,
    ) -> Result<CameraSnapshot> {
        let mut client = RtspClient::new(&camera.url);
        let data = client.capture_snapshot().await?;

        let timestamp = crate::timeutil::now_secs();
        let filename = format!("cam{}_{}.jpg", camera.id, timestamp);
        let file_path = format!("{}/{}", self.snapshot_dir.trim_end_matches('/'), filename);

        std::fs::write(&file_path, &data)
            .with_context(|| format!("Failed to write snapshot to {}", file_path))?;

        let file_size = data.len() as u64;

        let snapshot = CameraSnapshot {
            id: 0, // Will be assigned by storage
            camera_id: camera.id,
            timestamp,
            alarm_source: alarm_source.map(|s| s.to_string()),
            file_path: file_path.clone(),
            file_size,
        };

        // Update camera state
        if let Some(mut state) = self.cameras.get_mut(&camera.id) {
            state.last_snapshot = Some(snapshot.clone());
        }

        Ok(snapshot)
    }

    pub fn get_stream_url(&self, camera_id: i64) -> Option<String> {
        self.cameras
            .get(&camera_id)
            .map(|state| state.url.clone())
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    // ── RTSP URL Parsing Tests ──

    #[test]
    fn test_parse_rtsp_url_basic() {
        let (host, port, path, creds) =
            parse_rtsp_url("rtsp://192.168.1.100:554/stream").unwrap();
        assert_eq!(host, "192.168.1.100");
        assert_eq!(port, 554);
        assert_eq!(path, "/stream");
        assert!(creds.is_none());
    }

    #[test]
    fn test_parse_rtsp_url_default_port() {
        let (host, port, path, creds) =
            parse_rtsp_url("rtsp://10.0.0.1/live").unwrap();
        assert_eq!(host, "10.0.0.1");
        assert_eq!(port, 554);
        assert_eq!(path, "/live");
        assert!(creds.is_none());
    }

    #[test]
    fn test_parse_rtsp_url_with_credentials() {
        let (host, port, path, creds) =
            parse_rtsp_url("rtsp://admin:password@192.168.1.100:554/stream")
                .unwrap();
        assert_eq!(host, "192.168.1.100");
        assert_eq!(port, 554);
        assert_eq!(path, "/stream");
        let (user, pass) = creds.unwrap();
        assert_eq!(user, "admin");
        assert_eq!(pass, "password");
    }

    #[test]
    fn test_parse_rtsp_url_no_path() {
        let (host, port, path, _) =
            parse_rtsp_url("rtsp://192.168.1.100").unwrap();
        assert_eq!(host, "192.168.1.100");
        assert_eq!(port, 554);
        assert_eq!(path, "/");
    }

    #[test]
    fn test_parse_rtsp_url_custom_port() {
        let (host, port, _, _) =
            parse_rtsp_url("rtsp://192.168.1.100:8554/stream").unwrap();
        assert_eq!(host, "192.168.1.100");
        assert_eq!(port, 8554);
    }

    #[test]
    fn test_parse_rtsp_url_invalid_scheme() {
        assert!(parse_rtsp_url("http://example.com").is_err());
        assert!(parse_rtsp_url("rtsp:/example.com").is_err());
    }

    // ── RTSP Response Parsing Tests ──

    #[test]
    fn test_parse_rtsp_response_ok() {
        let raw = "RTSP/1.0 200 OK\r\nCSeq: 1\r\nSession: abc123\r\n\r\n";
        let resp = RtspResponse::parse(raw).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.status_text, "OK");
        assert_eq!(resp.header("CSeq"), Some("1"));
        assert_eq!(resp.header("Session"), Some("abc123"));
        assert!(resp.body.is_empty());
    }

    #[test]
    fn test_parse_rtsp_response_with_body() {
        let raw =
            "RTSP/1.0 200 OK\r\nCSeq: 2\r\nContent-Length: 10\r\n\r\nhello body";
        let resp = RtspResponse::parse(raw).unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.body, "hello body");
    }

    #[test]
    fn test_parse_rtsp_response_error() {
        let raw = "RTSP/1.0 404 Not Found\r\nCSeq: 3\r\n\r\n";
        let resp = RtspResponse::parse(raw).unwrap();
        assert_eq!(resp.status_code, 404);
        assert_eq!(resp.status_text, "Not Found");
    }

    #[test]
    fn test_parse_rtsp_response_empty() {
        assert!(RtspResponse::parse("").is_err());
    }

    // ── RTSP Request Construction Tests ──

    #[test]
    fn test_build_options_request() {
        let mut client = RtspClient::new("rtsp://192.168.1.100:554/stream");
        let req = client.build_request("OPTIONS", "");
        assert!(req.starts_with("OPTIONS rtsp://192.168.1.100:554/stream RTSP/1.0\r\n"));
        assert!(req.contains("CSeq: 1\r\n"));
        assert!(req.ends_with("\r\n\r\n"));
    }

    #[test]
    fn test_build_describe_request() {
        let mut client = RtspClient::new("rtsp://192.168.1.100:554/stream");
        let req = client.build_request("DESCRIBE", "Accept: application/sdp\r\n");
        assert!(req.starts_with("DESCRIBE rtsp://192.168.1.100:554/stream RTSP/1.0\r\n"));
        assert!(req.contains("Accept: application/sdp\r\n"));
        assert!(req.contains("CSeq: 1\r\n"));
    }

    #[test]
    fn test_build_setup_request_with_session() {
        let mut client = RtspClient::new("rtsp://192.168.1.100:554/stream");
        client.session_id = Some("sess123".to_string());
        let transport = "Transport: RTP/AVP;unicast;client_port=50000-50001\r\n";
        let req = client.build_request("SETUP", transport);
        assert!(req.contains("Session: sess123\r\n"));
        assert!(req.contains("Transport:"));
    }

    #[test]
    fn test_build_play_request() {
        let mut client = RtspClient::new("rtsp://192.168.1.100:554/stream");
        client.session_id = Some("sess123".to_string());
        // CSeq 1 already used above in session setup simulation
        let req = client.build_request("PLAY", "");
        assert!(req.starts_with("PLAY rtsp://"));
        assert!(req.contains("Session: sess123\r\n"));
    }

    #[test]
    fn test_build_teardown_request() {
        let mut client = RtspClient::new("rtsp://192.168.1.100:554/stream");
        client.session_id = Some("sess123".to_string());
        let req = client.build_request("TEARDOWN", "");
        assert!(req.starts_with("TEARDOWN rtsp://"));
    }

    #[test]
    fn test_cseq_increments() {
        let mut client = RtspClient::new("rtsp://192.168.1.100:554/stream");
        let req1 = client.build_request("OPTIONS", "");
        let req2 = client.build_request("OPTIONS", "");
        assert!(req1.contains("CSeq: 1\r\n"));
        assert!(req2.contains("CSeq: 2\r\n"));
    }

    #[test]
    fn test_auth_header_in_request() {
        let mut client =
            RtspClient::new("rtsp://admin:secret@192.168.1.100:554/stream");
        let req = client.build_request("OPTIONS", "");
        assert!(req.contains("Authorization: Basic "));
    }

    // ── SDP Parsing Tests ──

    #[test]
    fn test_parse_sdp_basic() {
        let sdp = "v=0\r\n\
             o=- 1234 1 IN IP4 192.168.1.100\r\n\
             s=RTSP Stream\r\n\
             m=video 0 RTP/AVP 96\r\n\
             a=rtpmap:96 JPEG/90000\r\n";
        let info = SdpInfo::parse(sdp).unwrap();
        assert_eq!(info.media_type, "video");
        assert_eq!(info.codec, "JPEG");
        assert!(info.is_mjpeg());
    }

    #[test]
    fn test_parse_sdp_h264() {
        let sdp = "v=0\r\n\
             m=video 0 RTP/AVP 97\r\n\
             a=rtpmap:97 H264/90000\r\n";
        let info = SdpInfo::parse(sdp).unwrap();
        assert_eq!(info.media_type, "video");
        assert_eq!(info.codec, "H264");
        assert!(!info.is_mjpeg());
    }

    #[test]
    fn test_parse_sdp_with_bandwidth() {
        let sdp = "v=0\r\n\
             m=video 0 RTP/AVP 96\r\n\
             b=AS:4000\r\n\
             a=rtpmap:96 JPEG/90000\r\n";
        let info = SdpInfo::parse(sdp).unwrap();
        assert_eq!(info.bandwidth, Some(4000));
    }

    #[test]
    fn test_parse_sdp_minimal() {
        let sdp = "v=0\r\ns=test\r\n";
        let info = SdpInfo::parse(sdp).unwrap();
        assert_eq!(info.media_type, "video");
        assert_eq!(info.codec, "unknown");
    }

    // ── JPEG extraction tests ──

    #[test]
    fn test_find_jpeg_in_data_with_markers() {
        let mut data = vec![0u8; 10];
        data[0] = 0x00;
        data[1] = 0xFF;
        data[2] = 0xD8; // SOI
        data[5] = 0xFF;
        data[6] = 0xD9; // EOI
        let result = find_jpeg_in_data(&data).unwrap();
        // Should extract from SOI to EOI
        assert_eq!(result[0], 0xFF);
        assert_eq!(result[1], 0xD8);
        assert_eq!(result[result.len() - 2], 0xFF);
        assert_eq!(result[result.len() - 1], 0xD9);
    }

    #[test]
    fn test_find_jpeg_no_markers() {
        let data = vec![0x00, 0x01, 0x02, 0x03];
        let result = find_jpeg_in_data(&data).unwrap();
        assert_eq!(result, data);
    }

    // ── Camera / Snapshot serialization tests ──

    #[test]
    fn test_camera_serialization() {
        let camera = Camera {
            id: 1,
            name: "Front Gate".to_string(),
            url: "rtsp://admin:pass@192.168.1.100:554/stream".to_string(),
            enabled: true,
            snapshot_on_alarm: true,
            retention_hours: 72,
            created_at: 1710000000,
        };
        let json = serde_json::to_string(&camera).unwrap();
        let parsed: Camera = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.name, "Front Gate");
        assert_eq!(parsed.url, "rtsp://admin:pass@192.168.1.100:554/stream");
        assert!(parsed.enabled);
        assert!(parsed.snapshot_on_alarm);
        assert_eq!(parsed.retention_hours, 72);
    }

    #[test]
    fn test_camera_snapshot_serialization() {
        let snapshot = CameraSnapshot {
            id: 42,
            camera_id: 1,
            timestamp: 1710000000,
            alarm_source: Some("temperature_critical".to_string()),
            file_path: "/snapshots/cam1_1710000000.jpg".to_string(),
            file_size: 2048,
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let parsed: CameraSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, 42);
        assert_eq!(parsed.camera_id, 1);
        assert_eq!(parsed.alarm_source, Some("temperature_critical".to_string()));
        assert_eq!(parsed.file_size, 2048);
    }

    #[test]
    fn test_camera_snapshot_no_alarm_source() {
        let snapshot = CameraSnapshot {
            id: 43,
            camera_id: 2,
            timestamp: 1710000000,
            alarm_source: None,
            file_path: "/snapshots/cam2_1710000000.jpg".to_string(),
            file_size: 1024,
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let parsed: CameraSnapshot = serde_json::from_str(&json).unwrap();
        assert!(parsed.alarm_source.is_none());
    }

    // ── Stream Proxy Tests ──

    #[test]
    fn test_stream_proxy_new() {
        let proxy = StreamProxy::new("/tmp/test_snapshots");
        assert!(proxy.cameras.is_empty());
        assert_eq!(proxy.snapshot_dir, "/tmp/test_snapshots");
    }

    #[tokio::test]
    async fn test_stream_proxy_add_remove_camera() {
        let proxy = StreamProxy::new("/tmp/test_snapshots_proxy");
        let camera = Camera {
            id: 1,
            name: "Test Camera".to_string(),
            url: "rtsp://192.168.1.100:554/stream".to_string(),
            enabled: true,
            snapshot_on_alarm: false,
            retention_hours: 24,
            created_at: 0,
        };
        proxy.add_camera(&camera).await;
        assert_eq!(proxy.get_stream_url(1), Some("rtsp://192.168.1.100:554/stream".to_string()));

        proxy.remove_camera(1).await;
        assert!(proxy.get_stream_url(1).is_none());
    }

    // ── License gate test ──

    #[test]
    fn test_rtsp_video_enterprise_only() {
        use crate::license::LicenseTier;
        assert!(!LicenseTier::Trial.rtsp_video());
        assert!(!LicenseTier::Starter.rtsp_video());
        assert!(!LicenseTier::Professional.rtsp_video());
        assert!(LicenseTier::Enterprise.rtsp_video());
    }
}
