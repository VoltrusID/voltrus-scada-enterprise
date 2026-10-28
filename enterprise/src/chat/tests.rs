use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::IntoResponse;
use serde_json::{Value, json};
use voltrus::edition::{ChatAlarm, ChatSender};

use super::*;

const TOKEN: &str = "123456789:AAEhBP0av28mG8Wv9oQxkXq1Zt2yVbcD3_4";

#[derive(Clone, Debug)]
struct Seen {
    path: String,
    authorization: Option<String>,
    body: Value,
}

#[derive(Clone, Default)]
struct Mock {
    seen: Arc<Mutex<Vec<Seen>>>,
    /// Responses handed out in order; when empty, `200 {"ok":true,"messages":[{"id":"m"}]}`.
    script: Arc<Mutex<VecDeque<(u16, Value, Vec<(&'static str, String)>)>>>,
}

impl Mock {
    fn push(&self, status: u16, body: Value) {
        self.script
            .lock()
            .unwrap()
            .push_back((status, body, vec![]));
    }
    fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
}

async fn handle(
    State(m): State<Mock>,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    m.seen.lock().unwrap().push(Seen {
        path: uri.path().to_string(),
        authorization: headers
            .get("authorization")
            .map(|v| v.to_str().unwrap().to_string()),
        body: serde_json::from_slice(&body).unwrap_or(Value::Null),
    });
    let next = m.script.lock().unwrap().pop_front();
    let (status, body, extra) = next.unwrap_or((
        200,
        json!({"ok": true, "messages": [{"id": "wamid.1"}]}),
        vec![],
    ));
    let mut resp = (StatusCode::from_u16(status).unwrap(), axum::Json(body)).into_response();
    for (k, v) in extra {
        resp.headers_mut().insert(k, v.parse().unwrap());
    }
    resp
}

async fn serve() -> (Mock, String) {
    let mock = Mock::default();
    let app = Router::new().fallback(handle).with_state(mock.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (mock, url)
}

fn fast() -> RetryPolicy {
    RetryPolicy {
        max_retries: 2,
        base_delay: Duration::from_millis(1),
        max_retry_after: Duration::from_secs(5),
    }
}

fn alarm() -> ChatAlarm {
    ChatAlarm {
        severity: "critical".into(),
        source_key: "pump_1:temp".into(),
        source_name: "Pump <1> & \"Co\"".into(),
        value: 91.5,
        threshold: 90.0,
        unit: "°C".into(),
        message: "Temp > limit & rising\nsecond line".into(),
        triggered_at: "2026-01-02T03:04:05Z".into(),
    }
}

fn telegram_config(chat_ids: &[&str]) -> String {
    json!({"bot_token": TOKEN, "chat_ids": chat_ids}).to_string()
}

fn whatsapp_config(patch: Value) -> String {
    let mut base = json!({
        "access_token": "EAAGtoken",
        "phone_number_id": "106540352242922",
        "recipients": ["+6281234567890"],
        "template_name": "alarm_notice",
        "language": "id",
        "parameters": ["severity", "source", "value", "message", "time"],
    });
    for (k, v) in patch.as_object().unwrap() {
        base[k] = v.clone();
    }
    base.to_string()
}

#[test]
fn telegram_text_escapes_html_in_every_alarm_field() {
    let text = telegram::format_text(&alarm());
    assert!(
        text.contains("<b>[CRITICAL] Pump &lt;1&gt; &amp; \"Co\"</b>"),
        "{text}"
    );
    assert!(
        text.contains("Temp &gt; limit &amp; rising\nsecond line"),
        "{text}"
    );
    assert!(
        text.contains("Value: <b>91.5 °C</b> (threshold 90)"),
        "{text}"
    );
    assert!(text.contains("<code>pump_1:temp</code>"), "{text}");
    // No raw alarm-controlled angle bracket survives outside our own tags.
    let stripped = text
        .replace("<b>", "")
        .replace("</b>", "")
        .replace("<code>", "")
        .replace("</code>", "");
    assert!(!stripped.contains('<') && !stripped.contains('>'), "{text}");
}

#[test]
fn telegram_text_stays_under_the_message_limit() {
    let mut a = alarm();
    a.message = "x".repeat(10_000);
    a.source_name = "n".repeat(10_000);
    a.source_key = "k".repeat(10_000);
    assert!(telegram::format_text(&a).chars().count() < 4096);
}

#[test]
fn whatsapp_parameters_follow_the_mapping_and_obey_meta_rules() {
    let cfg = voltrus::notification::chat::parse_whatsapp(&whatsapp_config(
        json!({"parameters": ["message", "severity", "value", "source", "time", "message"]}),
    ))
    .unwrap();
    let body = whatsapp::request_body(&cfg, "+6281234567890", &alarm());
    assert_eq!(body["to"], "6281234567890");
    assert_eq!(body["type"], "template");
    assert_eq!(body["template"]["name"], "alarm_notice");
    assert_eq!(body["template"]["language"]["code"], "id");
    let texts: Vec<&str> = body["template"]["components"][0]["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["text"].as_str().unwrap())
        .collect();
    assert_eq!(
        texts,
        [
            "Temp > limit & rising second line", // newline collapsed
            "CRITICAL",
            "91.5 °C",
            "Pump <1> & \"Co\"", // WhatsApp parameters are plain text, not HTML
            "2026-01-02T03:04:05Z",
            "Temp > limit & rising second line",
        ]
    );
    let mut empty = alarm();
    empty.message = "  \n ".into();
    assert_eq!(whatsapp::parameter_text("message", &empty), "-");
}

#[test]
fn whatsapp_without_parameters_sends_no_components() {
    let cfg =
        voltrus::notification::chat::parse_whatsapp(&whatsapp_config(json!({"parameters": []})))
            .unwrap();
    let body = whatsapp::request_body(&cfg, "+6281234567890", &alarm());
    assert!(body["template"].get("components").is_none());
}

#[tokio::test]
async fn telegram_posts_to_every_chat_and_never_leaks_the_token_in_the_body() {
    let (mock, url) = serve().await;
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send(
            "telegram",
            &telegram_config(&["111", "@plant_ops"]),
            &alarm(),
        )
        .await
        .unwrap();
    assert_eq!(out.len(), 2);
    assert!(out.iter().all(|d| d.error.is_none()));
    let seen = mock.seen();
    assert_eq!(seen[0].path, format!("/bot{TOKEN}/sendMessage"));
    assert_eq!(seen[0].body["chat_id"], "111");
    assert_eq!(seen[1].body["chat_id"], "@plant_ops");
    assert_eq!(seen[0].body["parse_mode"], "HTML");
    assert!(!seen[0].body.to_string().contains(TOKEN));
}

#[tokio::test]
async fn telegram_waits_out_a_429_retry_after_then_succeeds() {
    let (mock, url) = serve().await;
    mock.push(
        429,
        json!({"ok": false, "error_code": 429, "description": "Too Many Requests: retry after 0", "parameters": {"retry_after": 0}}),
    );
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send("telegram", &telegram_config(&["111"]), &alarm())
        .await
        .unwrap();
    assert!(out[0].error.is_none(), "{:?}", out[0].error);
    assert_eq!(mock.seen().len(), 2);
}

#[tokio::test]
async fn telegram_does_not_hold_the_task_for_a_long_rate_limit() {
    let (mock, url) = serve().await;
    mock.push(
        429,
        json!({"ok": false, "description": "Too Many Requests", "parameters": {"retry_after": 3600}}),
    );
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send("telegram", &telegram_config(&["111"]), &alarm())
        .await
        .unwrap();
    let err = out[0].error.as_deref().unwrap();
    assert!(err.contains("rate limited for 3600s"), "{err}");
    assert_eq!(mock.seen().len(), 1);
}

#[tokio::test]
async fn telegram_client_errors_are_not_retried_and_explained() {
    let (mock, url) = serve().await;
    mock.push(
        401,
        json!({"ok": false, "error_code": 401, "description": "Unauthorized"}),
    );
    mock.push(
        400,
        json!({"ok": false, "error_code": 400, "description": "Bad Request: chat not found"}),
    );
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send("telegram", &telegram_config(&["111", "222"]), &alarm())
        .await
        .unwrap();
    assert!(
        out[0]
            .error
            .as_deref()
            .unwrap()
            .contains("token is invalid")
    );
    assert!(
        out[1]
            .error
            .as_deref()
            .unwrap()
            .contains("Check the chat id")
    );
    assert_eq!(mock.seen().len(), 2);
}

#[tokio::test]
async fn telegram_retries_server_errors_with_backoff_then_gives_up() {
    let (mock, url) = serve().await;
    for _ in 0..3 {
        mock.push(502, json!({"ok": false, "description": "Bad Gateway"}));
    }
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send("telegram", &telegram_config(&["111"]), &alarm())
        .await
        .unwrap();
    assert!(
        out[0]
            .error
            .as_deref()
            .unwrap()
            .contains("gave up after 3 attempts")
    );
    assert_eq!(mock.seen().len(), 3);
}

#[tokio::test]
async fn telegram_network_errors_do_not_expose_the_token() {
    // Nothing listens on port 9.
    let chat = ChatChannels::new("http://127.0.0.1:9", "http://127.0.0.1:9", fast());
    let out = chat
        .send("telegram", &telegram_config(&["111"]), &alarm())
        .await
        .unwrap();
    let err = out[0].error.as_deref().unwrap();
    assert!(!err.contains(TOKEN), "{err}");
}

#[tokio::test]
async fn whatsapp_posts_the_template_with_a_bearer_token() {
    let (mock, url) = serve().await;
    let chat = ChatChannels::new(&url, &url, fast());
    let cfg = whatsapp_config(json!({"recipients": ["+6281234567890", "+14155550100"]}));
    let out = chat.send("whatsapp", &cfg, &alarm()).await.unwrap();
    assert!(out.iter().all(|d| d.error.is_none()));
    let seen = mock.seen();
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].path, "/v23.0/106540352242922/messages");
    assert_eq!(seen[0].authorization.as_deref(), Some("Bearer EAAGtoken"));
    assert_eq!(seen[1].body["to"], "14155550100");
    assert_eq!(seen[0].body["messaging_product"], "whatsapp");
}

#[tokio::test]
async fn whatsapp_reports_meta_errors_with_a_hint_and_does_not_retry_template_errors() {
    let (mock, url) = serve().await;
    mock.push(
        404,
        json!({"error": {"message": "(#132001) Template name does not exist in the translation", "type": "OAuthException", "code": 132001, "error_data": {"messaging_product": "whatsapp", "details": "template name (alarm_notice) does not exist in id"}}}),
    );
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send("whatsapp", &whatsapp_config(json!({})), &alarm())
        .await
        .unwrap();
    let err = out[0].error.as_deref().unwrap();
    assert!(err.contains("132001"), "{err}");
    assert!(err.contains("does not exist in id"), "{err}");
    assert!(err.contains("Hint: the template does not exist"), "{err}");
    assert_eq!(mock.seen().len(), 1);
}

#[tokio::test]
async fn whatsapp_retries_throttling_honouring_retry_after() {
    let (mock, url) = serve().await;
    mock.script.lock().unwrap().push_back((
        429,
        json!({"error": {"message": "Too many messages", "code": 130429}}),
        vec![("retry-after", "0".to_string())],
    ));
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send("whatsapp", &whatsapp_config(json!({})), &alarm())
        .await
        .unwrap();
    assert!(out[0].error.is_none(), "{:?}", out[0].error);
    assert_eq!(mock.seen().len(), 2);
}

#[tokio::test]
async fn whatsapp_expired_token_is_fatal() {
    let (mock, url) = serve().await;
    mock.push(
        401,
        json!({"error": {"message": "Error validating access token: Session has expired", "type": "OAuthException", "code": 190}}),
    );
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send("whatsapp", &whatsapp_config(json!({})), &alarm())
        .await
        .unwrap();
    assert!(
        out[0]
            .error
            .as_deref()
            .unwrap()
            .contains("invalid or expired")
    );
    assert_eq!(mock.seen().len(), 1);
}

#[tokio::test]
async fn whatsapp_success_without_a_message_id_is_an_error() {
    let (mock, url) = serve().await;
    mock.push(200, json!({"messaging_product": "whatsapp"}));
    let chat = ChatChannels::new(&url, &url, fast());
    let out = chat
        .send("whatsapp", &whatsapp_config(json!({})), &alarm())
        .await
        .unwrap();
    assert!(out[0].error.is_some());
}

#[tokio::test]
async fn invalid_config_is_rejected_before_any_request() {
    let (mock, url) = serve().await;
    let chat = ChatChannels::new(&url, &url, fast());
    assert!(chat.send("telegram", "{}", &alarm()).await.is_err());
    assert!(
        chat.send(
            "whatsapp",
            &whatsapp_config(json!({"recipients": ["0812"]})),
            &alarm()
        )
        .await
        .is_err()
    );
    assert!(chat.send("pager", "[]", &alarm()).await.is_err());
    assert!(mock.seen().is_empty());
}

#[test]
fn numbers_format_without_noise() {
    assert_eq!(format_number(90.0), "90");
    assert_eq!(format_number(91.5), "91.5");
    assert_eq!(format_number(0.0004), "0");
    assert_eq!(format_number(-12.3456), "-12.346");
}
