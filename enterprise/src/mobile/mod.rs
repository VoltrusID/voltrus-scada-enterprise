//! The push relay connector (protocol: `core/docs/mobile-relay.md`).
//!
//! A site licensed for Integrator or Enterprise keeps one WebSocket open to the
//! Voltrus push relay. Over it the site sends alarm pushes for the phones paired
//! with the Voltrus SCADA app, and answers the phones' requests the relay
//! forwards when a phone cannot reach the site directly. [`RelayConnector`] is
//! registered as Core's `MobilePushSender`.
//!
//! The connection is kept only while the licence includes `mobile_push`, an
//! activation file exists and this node is the active one (a standby of a hot
//! standby pair never connects).

mod forward;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock};
use std::time::{Duration, Instant};

use axum::Router;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::{Notify, mpsc, oneshot};
use tokio_tungstenite::tungstenite::{self, Message};
use voltrus::edition::{
    BoxFuture, MobileAlarmPush, MobilePushDelivery, MobilePushSender, MobileRelay,
};
use voltrus::license::LicenseStatus;
use voltrus::storage::Storage;

/// The relay used unless `VOLTRUS_PUSH_RELAY_URL` names another.
pub const DEFAULT_RELAY_URL: &str = "https://push.voltrus.id";

/// Settings key of the secret the relay issued to this site in a `hello`.
pub const SITE_SECRET_KEY: &str = "mobile_relay_site_secret";

/// How often and how long the connector waits.
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    /// `ping` frames, and the check that the licence and node role still allow
    /// the connection.
    pub ping_every: Duration,
    /// Nothing heard from the relay for this long: the connection is dead.
    pub silence_limit: Duration,
    /// First wait after a dropped connection; doubles up to `max_backoff`.
    pub first_backoff: Duration,
    pub max_backoff: Duration,
    /// Wait after the relay refused the connection (HTTP 4xx before the upgrade).
    pub refused_retry: Duration,
    /// Wait before checking again while the site may not connect (no licence,
    /// standby node). A licence change or takeover wakes it sooner.
    pub idle_check: Duration,
    /// How long a push waits for its `push_result`, and the hello for the relay.
    pub answer_timeout: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            ping_every: Duration::from_secs(30),
            silence_limit: Duration::from_secs(90),
            first_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            refused_retry: Duration::from_secs(3600),
            idle_check: Duration::from_secs(60),
            answer_timeout: Duration::from_secs(15),
        }
    }
}

type Pending = Arc<Mutex<HashMap<String, oneshot::Sender<Vec<MobilePushDelivery>>>>>;

/// The open connection, as the push sender and the request tasks use it.
#[derive(Clone)]
struct Link {
    out: mpsc::Sender<String>,
    pending: Pending,
}

#[derive(Default)]
struct LinkState {
    /// From the last `hello`, or worked out from the activation before that.
    site_id: Option<String>,
    link: Option<Link>,
}

/// Why one connection attempt ended.
enum Ended {
    /// The connection was up for `up_for` and then dropped.
    Dropped { why: String, up_for: Duration },
    /// The licence or node role no longer allows it, or the activation changed.
    Stopped,
}

enum ConnectError {
    /// The relay answered the upgrade with a client error.
    Refused {
        status: u16,
        body: String,
    },
    Failed(anyhow::Error),
}

impl<E: Into<anyhow::Error>> From<E> for ConnectError {
    fn from(e: E) -> Self {
        Self::Failed(e.into())
    }
}

/// The site's connection to the push relay, and Core's `MobilePushSender`.
pub struct RelayConnector {
    /// `https://…` as phones use it.
    base_url: String,
    storage: Storage,
    license: Arc<RwLock<LicenseStatus>>,
    /// The router requests run through; Core's `edition::app_router` unless set.
    router: OnceLock<Router>,
    timing: Timing,
    state: Mutex<LinkState>,
    wake: Notify,
    next_id: AtomicU64,
}

impl RelayConnector {
    pub fn new(
        base_url: &str,
        storage: Storage,
        license: Arc<RwLock<LicenseStatus>>,
        timing: Timing,
    ) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            storage,
            license,
            router: OnceLock::new(),
            timing,
            state: Mutex::new(LinkState::default()),
            wake: Notify::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// The relay named by `VOLTRUS_PUSH_RELAY_URL`, else [`DEFAULT_RELAY_URL`].
    pub fn relay_url_from_env() -> String {
        std::env::var("VOLTRUS_PUSH_RELAY_URL")
            .ok()
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| DEFAULT_RELAY_URL.to_string())
    }

    /// Run forwarded requests through `router` instead of the server's own
    /// (tests). Has no effect once set.
    pub fn use_router(&self, router: Router) {
        let _ = self.router.set(router);
    }

    /// Start the connection task. It idles while the site may not connect.
    pub fn spawn(self: &Arc<Self>) {
        let me = self.clone();
        tokio::spawn(async move { me.run().await });
    }

    /// Check the licence and node role again now (after an activation, a
    /// renewal, a takeover or a step down).
    pub fn wake(&self) {
        self.wake.notify_one();
    }

    /// Whether the connection is up.
    pub fn connected(&self) -> bool {
        self.lock().link.is_some()
    }

    fn lock(&self) -> MutexGuard<'_, LinkState> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn connect_url(&self) -> String {
        let ws = self
            .base_url
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1);
        format!("{ws}/v1/sites/connect")
    }

    fn router(&self) -> Option<Router> {
        match self.router.get() {
            Some(router) => Some(router.clone()),
            None => voltrus::edition::app_router().cloned(),
        }
    }

    /// Licence and node role allow a connection.
    fn allowed(&self) -> bool {
        voltrus::node_role::is_active()
            && self
                .license
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .tier()
                .mobile_push()
    }

    /// The activation file to present, when the site may connect.
    async fn activation(&self) -> Option<Vec<u8>> {
        if !self.allowed() {
            return None;
        }
        let blob = match crate::licensing::pair::borrowed_blob() {
            Some(blob) => blob,
            None => match self.storage.get_license_activation().await {
                Ok(row) => row?.activation_blob,
                Err(e) => {
                    tracing::warn!("Push relay: the licence activation could not be read: {e:#}");
                    return None;
                }
            },
        };
        let mut state = self.lock();
        if state.site_id.is_none() {
            state.site_id = site_id_of(&blob);
        }
        Some(blob)
    }

    async fn stored_secret(&self) -> Option<String> {
        match self.storage.get_settings(&[SITE_SECRET_KEY]).await {
            Ok(rows) => rows
                .into_iter()
                .next()
                .map(|s| s.value)
                .filter(|v| !v.is_empty()),
            Err(e) => {
                tracing::warn!("Push relay: the site secret could not be read: {e:#}");
                None
            }
        }
    }

    /// Sleep for `d`, or less when woken.
    async fn pause(&self, d: Duration) {
        tokio::select! {
            _ = tokio::time::sleep(d) => {}
            _ = self.wake.notified() => {}
        }
    }

    async fn run(self: Arc<Self>) {
        let mut backoff = self.timing.first_backoff;
        loop {
            let Some(activation) = self.activation().await else {
                self.pause(self.timing.idle_check).await;
                continue;
            };
            match self.connect_once(&activation).await {
                Ok(Ended::Stopped) => {
                    tracing::info!("Push relay: disconnected (licence or node role changed)");
                    backoff = self.timing.first_backoff;
                }
                Ok(Ended::Dropped { why, up_for }) => {
                    // A connection that stayed up starts the backoff again; one
                    // dropped at once (another node with the same licence keeps
                    // replacing it) keeps backing off.
                    if up_for >= self.timing.max_backoff {
                        backoff = self.timing.first_backoff;
                    }
                    tracing::warn!(
                        "Push relay: connection lost ({why}), retrying in {}s",
                        backoff.as_secs()
                    );
                    self.pause(backoff).await;
                    backoff = (backoff * 2).min(self.timing.max_backoff);
                }
                Err(ConnectError::Refused { status, body }) => {
                    tracing::warn!(
                        "Push relay refused the site ({status}: {body}), retrying in {} min",
                        self.timing.refused_retry.as_secs() / 60
                    );
                    self.pause(self.timing.refused_retry).await;
                }
                Err(ConnectError::Failed(e)) => {
                    tracing::warn!(
                        "Push relay: cannot connect ({e:#}), retrying in {}s",
                        backoff.as_secs()
                    );
                    self.pause(backoff).await;
                    backoff = (backoff * 2).min(self.timing.max_backoff);
                }
            }
        }
    }

    async fn handshake(
        &self,
        activation: &[u8],
    ) -> anyhow::Result<tungstenite::handshake::client::Request> {
        use tungstenite::client::IntoClientRequest;
        use tungstenite::http::HeaderValue;
        let mut request = self.connect_url().into_client_request()?;
        let headers = request.headers_mut();
        headers.insert(
            "X-Voltrus-Activation",
            HeaderValue::from_str(&base64::engine::general_purpose::STANDARD.encode(activation))?,
        );
        if let Some(secret) = self.stored_secret().await {
            headers.insert("X-Voltrus-Site-Secret", HeaderValue::from_str(&secret)?);
        }
        headers.insert(
            "X-Voltrus-Version",
            HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
        );
        Ok(request)
    }

    async fn connect_once(self: &Arc<Self>, activation: &[u8]) -> Result<Ended, ConnectError> {
        let request = self.handshake(activation).await?;
        let connecting = tokio_tungstenite::connect_async(request);
        let (ws, _) = match tokio::time::timeout(Duration::from_secs(30), connecting).await {
            Err(_) => return Err(anyhow::anyhow!("timed out").into()),
            Ok(Err(tungstenite::Error::Http(resp))) if resp.status().is_client_error() => {
                let body = resp
                    .body()
                    .as_deref()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
                    .unwrap_or_default();
                return Err(ConnectError::Refused {
                    status: resp.status().as_u16(),
                    body,
                });
            }
            Ok(Err(e)) => return Err(e.into()),
            Ok(Ok(ws)) => ws,
        };
        let (mut sink, mut stream) = ws.split();

        let hello = tokio::time::timeout(self.timing.answer_timeout, stream.next())
            .await
            .map_err(|_| anyhow::anyhow!("no hello from the relay"))?;
        let hello: Hello = match hello {
            Some(Ok(Message::Text(text))) => serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("bad hello from the relay: {e}"))?,
            Some(Ok(other)) => return Err(anyhow::anyhow!("expected hello, got {other:?}").into()),
            Some(Err(e)) => return Err(e.into()),
            None => return Err(anyhow::anyhow!("closed before hello").into()),
        };
        if hello.kind != "hello" {
            return Err(anyhow::anyhow!("expected hello, got '{}'", hello.kind).into());
        }
        if let Some(secret) = hello.site_secret.filter(|s| !s.is_empty()) {
            // Without it the next connection is refused: do not go on unsaved.
            self.storage
                .upsert_setting(SITE_SECRET_KEY, &secret)
                .await
                .map_err(|e| anyhow::anyhow!("the site secret could not be saved: {e:#}"))?;
        }

        let (out_tx, mut out_rx) = mpsc::channel::<String>(64);
        let link = Link {
            out: out_tx,
            pending: Arc::default(),
        };
        {
            let mut state = self.lock();
            state.site_id = Some(hello.site_id.clone());
            state.link = Some(link.clone());
        }
        tracing::info!("Push relay: connected as site {}", hello.site_id);

        let up_since = Instant::now();
        let ended = match self
            .serve(&mut sink, &mut stream, &mut out_rx, &link, activation)
            .await
        {
            Err(why) => Ended::Dropped {
                why,
                up_for: up_since.elapsed(),
            },
            Ok(()) => Ended::Stopped,
        };
        // Pushes waiting for an answer fail now (their senders are dropped).
        self.lock().link = None;
        link.pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        if matches!(ended, Ended::Stopped) {
            let _ = sink.send(Message::Close(None)).await;
        }
        Ok(ended)
    }

    async fn serve<S, R>(
        self: &Arc<Self>,
        sink: &mut S,
        stream: &mut R,
        out_rx: &mut mpsc::Receiver<String>,
        link: &Link,
        activation: &[u8],
    ) -> Result<(), String>
    where
        S: futures_util::Sink<Message, Error = tungstenite::Error> + Unpin,
        R: futures_util::Stream<Item = Result<Message, tungstenite::Error>> + Unpin,
    {
        let mut ping = tokio::time::interval(self.timing.ping_every);
        ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        ping.tick().await;
        let mut last_heard = Instant::now();
        loop {
            tokio::select! {
                msg = stream.next() => match msg {
                    Some(Ok(Message::Text(text))) => {
                        last_heard = Instant::now();
                        self.handle_frame(&text, link);
                    }
                    Some(Ok(Message::Close(frame))) => {
                        let why = frame
                            .map(|f| format!("closed by the relay: {} {}", u16::from(f.code), f.reason))
                            .unwrap_or_else(|| "closed by the relay".into());
                        return Err(why);
                    }
                    Some(Ok(_)) => last_heard = Instant::now(),
                    Some(Err(e)) => return Err(e.to_string()),
                    None => return Err("stream ended".into()),
                },
                Some(frame) = out_rx.recv() => {
                    if let Err(e) = sink.send(Message::text(frame)).await {
                        return Err(e.to_string());
                    }
                }
                _ = ping.tick() => {
                    if !self.allowed() {
                        return Ok(());
                    }
                    if last_heard.elapsed() > self.timing.silence_limit {
                        return Err("the relay went silent".into());
                    }
                    let frame = json!({"type": "ping"}).to_string();
                    if let Err(e) = sink.send(Message::text(frame)).await {
                        return Err(e.to_string());
                    }
                }
                _ = self.wake.notified() => {
                    // A new activation is presented on a new connection.
                    let current = self.activation().await;
                    if current.as_deref() != Some(activation) {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn handle_frame(self: &Arc<Self>, text: &str, link: &Link) {
        let frame: Incoming = match serde_json::from_str(text) {
            Ok(f) => f,
            Err(e) => {
                tracing::debug!("Push relay: unreadable frame: {e}");
                return;
            }
        };
        match frame {
            Incoming::PushResult { id, deliveries } => {
                let waiting = link
                    .pending
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .remove(&id);
                if let Some(tx) = waiting {
                    let deliveries = deliveries
                        .into_iter()
                        .map(|d| MobilePushDelivery {
                            device_id: d.device_id,
                            error: d.error,
                            unregistered: d.unregistered,
                        })
                        .collect();
                    let _ = tx.send(deliveries);
                }
            }
            Incoming::Unregistered { devices } => {
                let storage = self.storage.clone();
                tokio::spawn(async move {
                    for d in devices {
                        if let Err(e) = storage
                            .clear_mobile_push_token(d.device_id, &d.push_token)
                            .await
                        {
                            tracing::warn!(
                                "Push relay: push token of device {} not cleared: {e:#}",
                                d.device_id
                            );
                        }
                    }
                });
            }
            Incoming::Request(request) => {
                let router = self.router();
                let out = link.out.clone();
                tokio::spawn(async move {
                    let frame = forward::answer(router, request).await;
                    let _ = out.send(frame.to_string()).await;
                });
            }
            Incoming::Pong | Incoming::Hello => {}
            Incoming::Other => tracing::debug!("Push relay: frame of an unknown type ignored"),
        }
    }

    async fn push(&self, push: &MobileAlarmPush) -> anyhow::Result<Vec<MobilePushDelivery>> {
        let link = self
            .lock()
            .link
            .clone()
            .ok_or_else(|| anyhow::anyhow!("not connected to the push relay"))?;
        let id = format!("p{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        let frame = json!({"type": "push", "id": id, "push": outgoing(push)}).to_string();
        let (tx, rx) = oneshot::channel();
        link.pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id.clone(), tx);
        let forget = || {
            link.pending
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&id);
        };
        if link.out.send(frame).await.is_err() {
            forget();
            anyhow::bail!("the push relay connection closed");
        }
        match tokio::time::timeout(self.timing.answer_timeout, rx).await {
            Ok(Ok(deliveries)) => Ok(deliveries),
            Ok(Err(_)) => anyhow::bail!("the push relay connection closed before it answered"),
            Err(_) => {
                forget();
                anyhow::bail!(
                    "the push relay did not answer within {}s",
                    self.timing.answer_timeout.as_secs()
                )
            }
        }
    }
}

impl MobilePushSender for RelayConnector {
    fn relay(&self) -> Option<MobileRelay> {
        let state = self.lock();
        Some(MobileRelay {
            url: self.base_url.clone(),
            site_id: state.site_id.clone()?,
            connected: state.link.is_some(),
        })
    }

    fn send<'a>(
        &'a self,
        push: &'a MobileAlarmPush,
    ) -> BoxFuture<'a, anyhow::Result<Vec<MobilePushDelivery>>> {
        Box::pin(self.push(push))
    }
}

/// `push` as it leaves the site: without the alarm's details when the site
/// keeps them (`details` false).
fn outgoing(push: &MobileAlarmPush) -> MobileAlarmPush {
    let mut push = push.clone();
    if !push.details {
        push.source_key.clear();
        push.source_name.clear();
        push.message.clear();
        push.value = 0.0;
        push.unit.clear();
    }
    push
}

/// The relay's id for the site of `activation` (a `VOLACT\x01` file):
/// `site_id` when the activation has one, else the first 32 hex characters of
/// SHA-256 of the licence key.
fn site_id_of(activation: &[u8]) -> Option<String> {
    use sha2::Digest;
    #[derive(Deserialize)]
    struct Fields {
        license_key: String,
        #[serde(default)]
        site_id: Option<String>,
    }
    let json = activation.strip_prefix(b"VOLACT\x01".as_slice())?;
    let fields: Fields = serde_json::from_slice(json).ok()?;
    if let Some(id) = fields.site_id.filter(|id| !id.is_empty()) {
        return Some(id);
    }
    let digest = sha2::Sha256::digest(fields.license_key.as_bytes());
    Some(hex::encode(digest)[..32].to_string())
}

#[derive(Deserialize)]
struct Hello {
    #[serde(rename = "type")]
    kind: String,
    site_id: String,
    #[serde(default)]
    site_secret: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Incoming {
    PushResult {
        id: String,
        #[serde(default)]
        deliveries: Vec<Delivery>,
    },
    Unregistered {
        #[serde(default)]
        devices: Vec<UnregisteredDevice>,
    },
    Request(forward::RelayRequest),
    Pong,
    Hello,
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct Delivery {
    device_id: i64,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    unregistered: bool,
}

#[derive(Deserialize)]
struct UnregisteredDevice {
    device_id: i64,
    push_token: String,
}
