use anyhow::{Context, Result};

// ── RTSP URL Parsing ──

/// Parse an RTSP URL into (host, port, path, optional_credentials).
/// Returns `((host, port, path), Option<(username, password)>)`.
pub fn parse_rtsp_url(url: &str) -> Result<(String, u16, String, Option<(String, String)>)> {
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
        let port = if let Some(s) = rest.strip_prefix(':') {
            s.parse::<u16>().unwrap_or(554)
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
        let status_line = lines.next().context("Empty RTSP response")?;

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
            } else if let Some(rest) = line.strip_prefix("a=rtpmap:") {
                // e.g. a=rtpmap:96 JPEG/90000
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
