use super::*;
use crate::models::{Camera, CameraSnapshot};

// ── RTSP URL Parsing Tests ──

#[test]
fn test_parse_rtsp_url_basic() {
    let (host, port, path, creds) = parse_rtsp_url("rtsp://192.168.1.100:554/stream").unwrap();
    assert_eq!(host, "192.168.1.100");
    assert_eq!(port, 554);
    assert_eq!(path, "/stream");
    assert!(creds.is_none());
}

#[test]
fn test_parse_rtsp_url_default_port() {
    let (host, port, path, creds) = parse_rtsp_url("rtsp://10.0.0.1/live").unwrap();
    assert_eq!(host, "10.0.0.1");
    assert_eq!(port, 554);
    assert_eq!(path, "/live");
    assert!(creds.is_none());
}

#[test]
fn test_parse_rtsp_url_with_credentials() {
    let (host, port, path, creds) =
        parse_rtsp_url("rtsp://admin:password@192.168.1.100:554/stream").unwrap();
    assert_eq!(host, "192.168.1.100");
    assert_eq!(port, 554);
    assert_eq!(path, "/stream");
    let (user, pass) = creds.unwrap();
    assert_eq!(user, "admin");
    assert_eq!(pass, "password");
}

#[test]
fn test_parse_rtsp_url_no_path() {
    let (host, port, path, _) = parse_rtsp_url("rtsp://192.168.1.100").unwrap();
    assert_eq!(host, "192.168.1.100");
    assert_eq!(port, 554);
    assert_eq!(path, "/");
}

#[test]
fn test_parse_rtsp_url_custom_port() {
    let (host, port, _, _) = parse_rtsp_url("rtsp://192.168.1.100:8554/stream").unwrap();
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
    let raw = "RTSP/1.0 200 OK\r\nCSeq: 2\r\nContent-Length: 10\r\n\r\nhello body";
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
    let mut client = RtspClient::new("rtsp://admin:secret@192.168.1.100:554/stream");
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
    assert_eq!(
        parsed.alarm_source,
        Some("temperature_critical".to_string())
    );
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
    assert_eq!(
        proxy.get_stream_url(1),
        Some("rtsp://192.168.1.100:554/stream".to_string())
    );

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
