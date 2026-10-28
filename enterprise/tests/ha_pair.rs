//! Hot standby with two real nodes.
//!
//! Each node is a `voltrus-ha-test-node` process (the real add-on with a stub
//! Enterprise licence) with its own temp database, config and ports. The tests
//! talk to them over HTTP like a client and read and write their SQLite files
//! like an external process would.
//!
//! Run:  cargo test --features ha-test-node --test ha_pair -- --nocapture --test-threads=1
//! The measurements print as `MEASURED ...` lines. `--ignored` adds the trigger
//! overhead benchmark.

#![cfg(unix)]

use std::net::TcpListener as StdListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rusqlite::{Connection, OpenFlags};
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::sleep;

const SECRET: &str = "ha-test-shared-secret-0123456789abcdef0123456789";
const AUTH_SECRET: &str = "ha-test-auth-secret-0123456789abcdef";
const PASSWORD: &str = "ha-test-password-1";

/// The tests start real processes and measure time: run them one at a time.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn free_port() -> u16 {
    StdListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

// ── a node process ──

struct Node {
    name: &'static str,
    dir: PathBuf,
    http: u16,
    ha: u16,
    /// Where this node dials its peer: the peer's HA port, or a proxy in front of it.
    peer: String,
    role: &'static str,
    witness: Option<String>,
    ha_enabled: bool,
    extra_yaml: String,
    child: Option<Child>,
}

impl Node {
    fn new(root: &Path, name: &'static str, role: &'static str, peer: String) -> Self {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        Node {
            name,
            dir,
            http: free_port(),
            ha: free_port(),
            peer,
            role,
            witness: None,
            ha_enabled: true,
            extra_yaml: String::new(),
            child: None,
        }
    }

    fn db(&self) -> PathBuf {
        self.dir.join("data.db")
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.http)
    }

    fn config(&self) -> String {
        let mut y = format!(
            "server_port: {}\nserver_host: \"127.0.0.1\"\ndb_path: \"{}\"\npersist_interval_secs: 1\ncookie_secure: false\nauth_secret: \"{AUTH_SECRET}\"\n{}",
            self.http,
            self.db().display(),
            self.extra_yaml
        );
        if self.ha_enabled {
            y.push_str(&format!(
                "ha:\n  enabled: true\n  role: {}\n  node_id: {}\n  bind_address: \"127.0.0.1:{}\"\n  peer_address: \"{}\"\n  heartbeat_interval_ms: 200\n  failover_after_missed: 5\n  sync_interval_ms: 50\n  shared_secret: \"{SECRET}\"\n",
                self.role, self.name, self.ha, self.peer
            ));
            if let Some(w) = &self.witness {
                y.push_str(&format!(
                    "  witness:\n    kind: arbiter\n    address: \"{w}\"\n    timeout_ms: 1000\n"
                ));
            }
        }
        y
    }

    fn start(&mut self) {
        assert!(self.child.is_none(), "{} is already running", self.name);
        std::fs::write(self.dir.join("config.yaml"), self.config()).unwrap();
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("node.log"))
            .unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_voltrus-ha-test-node"))
            .arg("config.yaml")
            .current_dir(&self.dir)
            .env("RUST_LOG", "info")
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .spawn()
            .expect("start the test node");
        self.child = Some(child);
    }

    /// kill -9
    fn kill(&mut self) {
        if let Some(mut c) = self.child.take() {
            c.kill().ok();
            c.wait().ok();
        }
    }

    fn log(&self) -> String {
        std::fs::read_to_string(self.dir.join("node.log")).unwrap_or_default()
    }

    fn log_len(&self) -> usize {
        self.log().len()
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.kill();
        if std::thread::panicking() {
            let log = self.log();
            let tail: Vec<&str> = log.lines().rev().take(60).collect();
            eprintln!(
                "---- last log lines of {} ----\n{}",
                self.name,
                tail.into_iter().rev().collect::<Vec<_>>().join("\n")
            );
        }
    }
}

// ── HTTP ──

#[derive(Clone)]
struct Api {
    http: reqwest::Client,
    cookie: String,
}

impl Api {
    async fn login(node: &Node) -> Api {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let resp = http
            .post(node.url("/api/v1/auth/login"))
            .json(&json!({"username": "admin", "password": PASSWORD}))
            .send()
            .await
            .expect("login request");
        assert_eq!(resp.status(), 200, "login on {}", node.name);
        let cookie = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|c| c.starts_with("voltrus_token="))
            .expect("session cookie")
            .split(';')
            .next()
            .unwrap()
            .to_string();
        Api { http, cookie }
    }

    async fn call(
        &self,
        method: reqwest::Method,
        node: &Node,
        path: &str,
        body: Option<Value>,
    ) -> Option<(u16, Value, Option<String>)> {
        let mut req = self
            .http
            .request(method, node.url(path))
            .header("Cookie", &self.cookie);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req.send().await.ok()?;
        let status = resp.status().as_u16();
        let role_header = resp
            .headers()
            .get("x-voltrus-node-role")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let v = resp.json::<Value>().await.unwrap_or(Value::Null);
        Some((status, v, role_header))
    }

    async fn get(&self, node: &Node, path: &str) -> Option<(u16, Value)> {
        self.call(reqwest::Method::GET, node, path, None)
            .await
            .map(|(s, v, _)| (s, v))
    }

    async fn post(
        &self,
        node: &Node,
        path: &str,
        body: Value,
    ) -> Option<(u16, Value, Option<String>)> {
        self.call(reqwest::Method::POST, node, path, Some(body))
            .await
    }

    async fn status(&self, node: &Node) -> Option<Value> {
        match self.get(node, "/api/v1/ha/status").await {
            Some((200, v)) => Some(v),
            _ => None,
        }
    }
}

async fn wait_until<T, F, Fut>(what: &str, within: Duration, mut f: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    let start = Instant::now();
    loop {
        if let Some(v) = f().await {
            return v;
        }
        assert!(
            start.elapsed() < within,
            "timed out after {within:?} waiting for: {what}"
        );
        sleep(Duration::from_millis(40)).await;
    }
}

/// Waits until the node answers HTTP. Returns the `x-voltrus-node-role` header of
/// its first answer: `standby` when it came up passive, `None` when active.
async fn wait_up(n: &Node) -> Option<String> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    wait_until(
        &format!("{} to answer HTTP", n.name),
        Duration::from_secs(60),
        || async {
            let r = http
                .get(n.url("/api/v1/health"))
                .send()
                .await
                .ok()
                .filter(|r| r.status().is_success())?;
            Some(
                r.headers()
                    .get("x-voltrus-node-role")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string),
            )
        },
    )
    .await
}

fn role_of(s: &Value) -> &str {
    s["role"].as_str().unwrap_or("?")
}

fn is_active(s: &Value) -> bool {
    s["active"].as_bool().unwrap_or(false)
}

async fn wait_role(api: &Api, n: &Node, role: &str, within: Duration) -> Value {
    wait_until(&format!("{} to be {role}", n.name), within, || async {
        api.status(n)
            .await
            .filter(|s| role_of(s) == role && (role != "primary" || is_active(s)))
    })
    .await
}

/// A standby that follows and has applied everything.
async fn wait_streaming(api: &Api, standby: &Node) -> Value {
    wait_until(
        &format!("{} to stream", standby.name),
        Duration::from_secs(60),
        || async {
            api.status(standby).await.filter(|s| {
                role_of(s) == "standby"
                    && s["sync_state"] == "streaming"
                    && s["peer_status"] == "connected"
                    && s["can_take_over"] == true
            })
        },
    )
    .await
}

// ── the databases, as an outside process sees them ──

async fn seed_admin(db: &Path) {
    let storage = voltrus::storage::Storage::new(db.to_str().unwrap()).unwrap();
    let hash = voltrus::api::auth::hash_password(PASSWORD);
    storage
        .create_user("admin", &hash, "", "admin")
        .await
        .unwrap();
}

fn ro(db: &Path) -> Option<Connection> {
    Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
}

/// One number from a query, integer or REAL (`MAX(value)` is REAL).
fn scalar(db: &Path, sql: &str) -> Option<i64> {
    let v = ro(db)?
        .query_row(sql, [], |r| r.get::<_, Option<f64>>(0))
        .ok()
        .flatten()?;
    Some(v as i64)
}

fn exec(db: &Path, sql: &str) {
    let c = Connection::open(db).unwrap();
    c.busy_timeout(Duration::from_secs(5)).unwrap();
    c.execute_batch(sql).unwrap();
}

/// Counts and sums of the tables the tests touch.
fn fingerprint(db: &Path) -> Option<String> {
    let c = ro(db)?;
    let mut out = Vec::new();
    for sql in [
        "SELECT COUNT(*) FROM telemetry",
        "SELECT COALESCE(SUM(value), 0) FROM telemetry",
        "SELECT COUNT(*) FROM modbus_devices",
        "SELECT COUNT(*) FROM modbus_registers",
        "SELECT COUNT(*) FROM alarms",
        "SELECT COUNT(*) FROM users",
        "SELECT COUNT(*) FROM audit_log",
    ] {
        out.push(
            c.query_row(sql, [], |r| r.get::<_, f64>(0))
                .ok()?
                .to_string(),
        );
    }
    Some(out.join("/"))
}

async fn wait_same(a: &Node, b: &Node, what: &str) {
    wait_until(what, Duration::from_secs(30), || async {
        let (fa, fb) = (fingerprint(&a.db())?, fingerprint(&b.db())?);
        (fa == fb).then_some(())
    })
    .await
}

/// A thread inserting one telemetry row every 2 ms into a database file, like
/// the persistence task of a busy node.
struct Writer {
    stop: Arc<AtomicBool>,
    counter: Arc<AtomicI64>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Writer {
    fn start(db: PathBuf, device: &'static str) -> Writer {
        let stop = Arc::new(AtomicBool::new(false));
        let counter = Arc::new(AtomicI64::new(0));
        let (s, c) = (stop.clone(), counter.clone());
        let handle = std::thread::spawn(move || {
            let conn = Connection::open(db).unwrap();
            conn.busy_timeout(Duration::from_secs(5)).unwrap();
            let mut i = 0i64;
            while !s.load(Ordering::SeqCst) {
                i += 1;
                let ts = voltrus::timeutil::now_millis();
                if conn
                    .execute(
                        "INSERT INTO telemetry (device_id, timestamp, metric, value) VALUES (?1, ?2, 'seq', ?3)",
                        rusqlite::params![device, ts, i as f64],
                    )
                    .is_err()
                {
                    break;
                }
                c.store(i, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(2));
            }
        });
        Writer {
            stop,
            counter,
            handle: Some(handle),
        }
    }

    fn committed(&self) -> i64 {
        self.counter.load(Ordering::SeqCst)
    }

    fn stop(mut self) -> i64 {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            h.join().ok();
        }
        self.committed()
    }
}

/// Counts the moments two nodes report themselves active at once.
struct DualActiveMonitor {
    stop: Arc<AtomicBool>,
    violations: Arc<AtomicU32>,
    samples: Arc<AtomicU32>,
    handle: JoinHandle<()>,
}

impl DualActiveMonitor {
    fn start(api: Api, a_http: u16, b_http: u16) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let violations = Arc::new(AtomicU32::new(0));
        let samples = Arc::new(AtomicU32::new(0));
        let (s, v, n) = (stop.clone(), violations.clone(), samples.clone());
        let handle = tokio::spawn(async move {
            let status = |port: u16| {
                let api = api.clone();
                async move {
                    let r = api
                        .http
                        .get(format!("http://127.0.0.1:{port}/api/v1/ha/status"))
                        .header("Cookie", &api.cookie)
                        .timeout(Duration::from_millis(400))
                        .send()
                        .await
                        .ok()?;
                    r.json::<Value>().await.ok()
                }
            };
            while !s.load(Ordering::SeqCst) {
                let (a, b) = tokio::join!(status(a_http), status(b_http));
                if let (Some(a), Some(b)) = (a, b) {
                    n.fetch_add(1, Ordering::SeqCst);
                    if is_active(&a) && is_active(&b) {
                        v.fetch_add(1, Ordering::SeqCst);
                    }
                }
                sleep(Duration::from_millis(30)).await;
            }
        });
        Self {
            stop,
            violations,
            samples,
            handle,
        }
    }

    async fn finish(self) -> (u32, u32) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.handle.await;
        (
            self.violations.load(Ordering::SeqCst),
            self.samples.load(Ordering::SeqCst),
        )
    }
}

/// A node that is not active must refuse every write, including the device write endpoint.
async fn assert_passive(api: &Api, n: &Node) {
    for (path, body) in [
        ("/api/v1/tags/1/write", json!({"value": 1.0})),
        (
            "/api/v1/modbus-devices",
            json!({"id": "nope", "name": "x", "ip": "127.0.0.1", "registers": []}),
        ),
    ] {
        let (status, v, role) = api.post(n, path, body).await.expect("passive node answers");
        assert_eq!(
            status, 503,
            "{}: POST {path} must be refused, got {status} {v}",
            n.name
        );
        assert_eq!(v["code"], "standby_read_only");
        assert_eq!(role.as_deref(), Some("standby"));
    }
    let (status, _) = api.get(n, "/api/v1/modbus-devices").await.unwrap();
    assert_eq!(status, 200, "{} still serves reads", n.name);
}

fn device_json(id: &str) -> Value {
    json!({
        "id": id, "name": format!("Device {id}"), "ip": "127.0.0.1", "port": 1,
        "interval_ms": 1000,
        "registers": [{"name": "temp", "address": 0, "register_type": "float32"}]
    })
}

async fn device_ids(api: &Api, n: &Node) -> Option<Vec<String>> {
    let (s, v) = api.get(n, "/api/v1/modbus-devices").await?;
    (s == 200).then(|| {
        v.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|d| d["id"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    })
}

async fn poller_ids(api: &Api, n: &Node) -> Vec<String> {
    match api.get(n, "/api/v1/modbus-devices/status").await {
        Some((200, v)) => v
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|d| d["device_id"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        _ => vec![],
    }
}

fn measured(line: String) {
    println!("MEASURED {line}");
}

// ───────────────────────────── the lifecycle ─────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn config_data_failover_rejoin_and_switchover() {
    let _serial = SERIAL.lock().await;
    let root = tempfile::tempdir().unwrap();
    let mut a = Node::new(root.path(), "node-a", "primary", String::new());
    let mut b = Node::new(
        root.path(),
        "node-b",
        "standby",
        format!("127.0.0.1:{}", a.ha),
    );
    a.peer = format!("127.0.0.1:{}", b.ha);

    // ── start the pair ──
    seed_admin(&a.db()).await;
    a.start();
    wait_up(&a).await;
    let api = Api::login(&a).await;
    let first = wait_role(&api, &a, "primary", Duration::from_secs(30)).await;
    assert_eq!(first["epoch"], 1);
    assert!(first["failover_ts"].is_null());

    let (status, _, _) = api
        .post(&a, "/api/v1/modbus-devices", device_json("plc1"))
        .await
        .unwrap();
    assert!(
        (200..300).contains(&status),
        "create device on the primary: {status}"
    );
    assert!(
        poller_ids(&api, &a).await.contains(&"plc1".to_string()),
        "the primary polls"
    );

    b.start();
    wait_up(&b).await;
    let sb = wait_streaming(&api, &b).await;
    assert_eq!(sb["epoch"], 1);
    assert_eq!(sb["active"], false);
    // The session cookie of the primary works on the standby: users and the signing secret are shared.
    assert_eq!(
        device_ids(&api, &b).await.unwrap(),
        vec!["plc1"],
        "the first full sync copied the device"
    );
    assert!(
        poller_ids(&api, &b).await.is_empty(),
        "the standby does not poll devices"
    );
    assert_passive(&api, &b).await;
    wait_same(
        &a,
        &b,
        "the standby to match the primary after the first sync",
    )
    .await;

    // ── config change on the primary shows up on the standby ──
    let t = Instant::now();
    let (s, _, _) = api
        .post(&a, "/api/v1/modbus-devices", device_json("plc2"))
        .await
        .unwrap();
    assert!((200..300).contains(&s));
    wait_until("plc2 on the standby", Duration::from_secs(10), || async {
        device_ids(&api, &b)
            .await
            .filter(|d| d.contains(&"plc2".to_string()))
    })
    .await;
    measured(format!(
        "config change visible on the standby after {} ms",
        t.elapsed().as_millis()
    ));
    let (s, _) = api
        .call(
            reqwest::Method::DELETE,
            &a,
            "/api/v1/modbus-devices/plc2",
            None,
        )
        .await
        .map(|(s, v, _)| (s, v))
        .unwrap();
    assert!((200..300).contains(&s), "delete device: {s}");
    wait_until(
        "plc2 deleted on the standby",
        Duration::from_secs(10),
        || async {
            device_ids(&api, &b)
                .await
                .filter(|d| !d.contains(&"plc2".to_string()))
        },
    )
    .await;

    // ── telemetry and alarms replicate ──
    {
        let rt_db = a.db();
        let rows: Vec<(String, i64, String, f64)> = (0..3000)
            .map(|i| ("plc1".into(), 1_000 + i, "temp".into(), i as f64))
            .collect();
        let storage = voltrus::storage::Storage::new(rt_db.to_str().unwrap()).unwrap();
        storage.insert_batch(&rows).await.unwrap();
        storage
            .create_alarm_with_type("temp", "critical", 99.0, 90.0, "temp too high", 0.0, "high")
            .await
            .unwrap();
    }
    wait_until(
        "telemetry and the alarm on the standby",
        Duration::from_secs(15),
        || async {
            let n = scalar(
                &b.db(),
                "SELECT COUNT(*) FROM telemetry WHERE device_id = 'plc1'",
            )?;
            let al = scalar(&b.db(), "SELECT COUNT(*) FROM alarms")?;
            (n == 3000 && al == 1).then_some(())
        },
    )
    .await;
    assert!(
        scalar(
            &b.db(),
            "SELECT COUNT(*) FROM audit_log WHERE action LIKE '%modbus_device%'"
        )
        .unwrap_or(0)
            >= 1,
        "the audit stream replicates"
    );
    wait_same(&a, &b, "databases to match").await;

    // ── no dual-active moment from here on ──
    let monitor = DualActiveMonitor::start(api.clone(), a.http, b.http);

    // ── kill -9 the primary under write load: failover time and data-loss window ──
    let writer = Writer::start(a.db(), "loss1");
    sleep(Duration::from_millis(1500)).await;
    wait_same_prefix_caught_up(&a, &b).await;
    let t_kill = Instant::now();
    a.kill();
    let at_kill = writer.committed();
    let promoted = wait_role(&api, &b, "primary", Duration::from_secs(20)).await;
    let failover_ms = t_kill.elapsed().as_millis();
    wait_until(
        "writes accepted on the new primary",
        Duration::from_secs(20),
        || async {
            let (s, _, _) = api
                .post(&b, "/api/v1/modbus-devices", device_json("plc3"))
                .await?;
            (s < 300).then_some(())
        },
    )
    .await;
    let writable_ms = t_kill.elapsed().as_millis();
    let tail = writer.stop();
    let replicated = scalar(
        &b.db(),
        "SELECT COALESCE(MAX(value), 0) FROM telemetry WHERE device_id = 'loss1'",
    )
    .unwrap();
    let lost = at_kill - replicated;
    measured(format!(
        "kill -9 primary: standby active after {failover_ms} ms, accepting writes after {writable_ms} ms (failover_after_ms={})",
        promoted["failover_after_ms"]
    ));
    measured(format!(
        "data loss at kill -9 (writes every 2 ms): {at_kill} rows committed, {replicated} replicated, {lost} lost = about {} ms of writes (sync_interval 50 ms); rows written after the kill: {}",
        lost * 2,
        tail - at_kill
    ));
    assert!(failover_ms < 6000, "failover took {failover_ms} ms");
    assert!(
        lost <= 150,
        "lost {lost} rows, more than the replication window allows"
    );
    assert_eq!(promoted["epoch"], 2, "a new term");
    assert_eq!(
        promoted["failover_reason"]
            .as_str()
            .map(|r| r.contains("no answer")),
        Some(true)
    );
    assert!(
        poller_ids(&api, &b).await.contains(&"plc1".to_string()),
        "the new primary started the device pollers"
    );
    assert!(
        device_ids(&api, &b)
            .await
            .unwrap()
            .contains(&"plc1".to_string()),
        "config survived"
    );
    assert_eq!(
        scalar(
            &b.db(),
            "SELECT COUNT(*) FROM telemetry WHERE device_id = 'plc1'"
        ),
        Some(3000),
        "history survived"
    );

    // ── the old primary comes back and must not be active ──
    a.start();
    let first = wait_up(&a).await;
    assert_eq!(
        first.as_deref(),
        Some("standby"),
        "a restarted old primary must come up passive, not resume"
    );
    let sa = wait_streaming(&api, &a).await;
    assert_passive(&api, &a).await;
    assert_eq!(sa["epoch"], 2);
    assert!(
        poller_ids(&api, &a).await.is_empty(),
        "the old primary stopped polling"
    );
    wait_same(
        &a,
        &b,
        "the old primary to take a full sync from the new one",
    )
    .await;
    assert_eq!(
        scalar(
            &a.db(),
            "SELECT COALESCE(MAX(value), 0) FROM telemetry WHERE device_id = 'loss1'"
        ),
        Some(replicated),
        "the rows the new primary never received are not kept on the old one"
    );

    // ── force-sync really resyncs ──
    exec(&a.db(), "DELETE FROM modbus_devices");
    sleep(Duration::from_millis(600)).await;
    assert_eq!(
        scalar(&a.db(), "SELECT COUNT(*) FROM modbus_devices"),
        Some(0),
        "incremental sync does not repair a tampered standby"
    );
    let (s, v, _) = api
        .post(&b, "/api/v1/ha/force-sync?wait_secs=30", json!({}))
        .await
        .unwrap();
    assert_eq!(s, 200, "force-sync: {v}");
    assert_eq!(v["status"], "synced");
    assert!(
        v["report"]["rows"].as_u64().unwrap() > 3000,
        "the full sync sent the data: {v}"
    );
    assert!(
        scalar(&a.db(), "SELECT COUNT(*) FROM modbus_devices").unwrap() >= 1,
        "force-sync restored the standby"
    );
    wait_same(&a, &b, "after force-sync").await;
    // A standby asks its primary: same result.
    exec(&a.db(), "DELETE FROM modbus_devices");
    let (s, v, _) = api
        .post(&a, "/api/v1/ha/force-sync?wait_secs=30", json!({}))
        .await
        .unwrap();
    assert_eq!(s, 200, "force-sync from the standby: {v}");
    wait_same(&a, &b, "after force-sync requested by the standby").await;
    wait_streaming(&api, &a).await;

    // ── controlled switchover from the primary: nothing is lost, no full sync ──
    {
        let rows: Vec<(String, i64, String, f64)> = (0..500)
            .map(|i| ("sw".into(), 9_000 + i, "m".into(), i as f64))
            .collect();
        let storage = voltrus::storage::Storage::new(b.db().to_str().unwrap()).unwrap();
        storage.insert_batch(&rows).await.unwrap();
    }
    let mark = a.log_len();
    let t = Instant::now();
    let (s, v, _) = api
        .post(&b, "/api/v1/ha/force-failover?wait_secs=30", json!({}))
        .await
        .unwrap();
    assert_eq!(s, 200, "switchover: {v}");
    assert_eq!(v["status"], "failover_complete");
    assert_eq!(v["new_role"], "standby");
    measured(format!(
        "controlled switchover (primary asked): {} ms, reported {} ms",
        t.elapsed().as_millis(),
        v["duration_ms"]
    ));
    let pa = wait_role(&api, &a, "primary", Duration::from_secs(20)).await;
    assert_eq!(pa["epoch"], 3);
    assert_eq!(
        scalar(
            &a.db(),
            "SELECT COUNT(*) FROM telemetry WHERE device_id = 'sw'"
        ),
        Some(500),
        "nothing lost in a controlled switchover"
    );
    let sb = wait_streaming(&api, &b).await;
    assert_eq!(sb["epoch"], 3);
    assert_passive(&api, &b).await;
    assert!(poller_ids(&api, &b).await.is_empty() && !poller_ids(&api, &a).await.is_empty());
    let b_log = b.log();
    assert!(
        b_log.contains("incremental sync from change"),
        "the old primary follows incrementally"
    );
    assert!(
        !a.log()[mark..].contains("full sync started")
            && !b
                .log()
                .rsplit("handed the primary role over")
                .next()
                .unwrap_or("")
                .contains("full sync started"),
        "no full sync after a clean switchover"
    );
    wait_same(&a, &b, "after the switchover").await;

    // ── and back, requested on the standby ──
    let t = Instant::now();
    let (s, v, _) = api
        .post(&b, "/api/v1/ha/force-failover?wait_secs=30", json!({}))
        .await
        .unwrap();
    assert_eq!(s, 200, "switchover requested by the standby: {v}");
    assert_eq!(v["new_role"], "primary");
    measured(format!(
        "controlled switchover (standby asked): {} ms",
        t.elapsed().as_millis()
    ));
    wait_role(&api, &b, "primary", Duration::from_secs(10)).await;
    wait_role(&api, &a, "standby", Duration::from_secs(10)).await;
    wait_streaming(&api, &a).await;

    let (violations, samples) = monitor.finish().await;
    measured(format!("dual-active samples: {violations} of {samples}"));
    assert!(samples > 100, "the monitor ran ({samples} samples)");
    assert_eq!(violations, 0, "two nodes reported active at the same time");
}

/// The standby holds everything the primary committed so far (the primary keeps writing).
async fn wait_same_prefix_caught_up(a: &Node, b: &Node) {
    wait_until(
        "the standby to be caught up",
        Duration::from_secs(10),
        || async {
            let pa = scalar(
                &a.db(),
                "SELECT COALESCE(MAX(value), 0) FROM telemetry WHERE device_id = 'loss1'",
            )?;
            let pb = scalar(
                &b.db(),
                "SELECT COALESCE(MAX(value), 0) FROM telemetry WHERE device_id = 'loss1'",
            )?;
            (pa - pb < 60 && pb > 0).then_some(())
        },
    )
    .await
}

// ───────────────────────────── the witness ─────────────────────────────

/// A TCP forwarder we can cut.
struct Proxy {
    port: u16,
    blocked: Arc<AtomicBool>,
    conns: Arc<Mutex<Vec<tokio::task::AbortHandle>>>,
    accept: JoinHandle<()>,
}

impl Proxy {
    async fn start(upstream: String) -> Proxy {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let blocked = Arc::new(AtomicBool::new(false));
        let conns: Arc<Mutex<Vec<tokio::task::AbortHandle>>> = Arc::default();
        let (bl, cs) = (blocked.clone(), conns.clone());
        let accept = tokio::spawn(async move {
            loop {
                let Ok((mut client, _)) = listener.accept().await else {
                    continue;
                };
                if bl.load(Ordering::SeqCst) {
                    continue; // dropped: the dialer sees a reset
                }
                let up = upstream.clone();
                let h = tokio::spawn(async move {
                    if let Ok(mut server) = TcpStream::connect(up).await {
                        let _ = tokio::io::copy_bidirectional(&mut client, &mut server).await;
                    }
                });
                cs.lock().unwrap().push(h.abort_handle());
            }
        });
        Proxy {
            port,
            blocked,
            conns,
            accept,
        }
    }

    fn block(&self) {
        self.blocked.store(true, Ordering::SeqCst);
        for h in self.conns.lock().unwrap().drain(..) {
            h.abort();
        }
    }

    fn unblock(&self) {
        self.blocked.store(false, Ordering::SeqCst);
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {
        self.accept.abort();
        for h in self.conns.lock().unwrap().drain(..) {
            h.abort();
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_partition_never_produces_two_primaries_with_an_arbiter() {
    let _serial = SERIAL.lock().await;
    let root = tempfile::tempdir().unwrap();

    // The arbiter runs in this process.
    let wport = free_port();
    let state = root.path().join("witness.json");
    let start_arbiter = |state: PathBuf| async move {
        let l = TcpListener::bind(("127.0.0.1", wport)).await.unwrap();
        tokio::spawn(voltrus_enterprise::ha::witness::run_arbiter_on(
            l, SECRET, state,
        ))
    };
    let arbiter = start_arbiter(state.clone()).await;

    let mut a = Node::new(root.path(), "node-a", "primary", String::new());
    let mut b = Node::new(root.path(), "node-b", "standby", String::new());
    let proxy_to_b = Proxy::start(format!("127.0.0.1:{}", b.ha)).await; // A dials this
    let proxy_to_a = Proxy::start(format!("127.0.0.1:{}", a.ha)).await; // B dials this
    a.peer = format!("127.0.0.1:{}", proxy_to_b.port);
    b.peer = format!("127.0.0.1:{}", proxy_to_a.port);
    for n in [&mut a, &mut b] {
        n.witness = Some(format!("127.0.0.1:{wport}"));
    }

    seed_admin(&a.db()).await;
    a.start();
    wait_up(&a).await;
    let api = Api::login(&a).await;
    let pa = wait_role(&api, &a, "primary", Duration::from_secs(30)).await;
    assert_eq!(
        pa["epoch"], 2,
        "the arbiter hands out the epoch: the first term is its second grant here"
    );
    assert_eq!(pa["witness"]["kind"], "arbiter");
    b.start();
    wait_up(&b).await;
    wait_streaming(&api, &b).await;
    let monitor = DualActiveMonitor::start(api.clone(), a.http, b.http);

    // 1. The link between the nodes is cut; both can still reach the arbiter.
    proxy_to_b.block();
    proxy_to_a.block();
    sleep(Duration::from_secs(5)).await; // well past the 1 s failover time
    let sa = api.status(&a).await.unwrap();
    let sb = api.status(&b).await.unwrap();
    assert_eq!(role_of(&sa), "primary", "the primary keeps its lease: {sa}");
    assert_eq!(
        role_of(&sb),
        "standby",
        "the standby must not take over while the lease is held: {sb}"
    );
    assert!(
        sb["promotion_blocked"]
            .as_str()
            .is_some_and(|m| m.contains("lease")),
        "it says why: {}",
        sb["promotion_blocked"]
    );
    assert_eq!(sa["epoch"], sb["epoch"], "no new term was started");
    measured("partition with both nodes reaching the arbiter: standby stayed standby for 5 s (failover time 1 s)".into());

    // The link heals.
    proxy_to_b.unblock();
    proxy_to_a.unblock();
    let healed = wait_streaming(&api, &b).await;
    assert!(
        healed["promotion_blocked"].is_null(),
        "the stale warning is cleared once the primary answers"
    );

    // 2. The primary loses the arbiter: it fences itself. Nobody else may act either.
    arbiter.abort();
    let fenced = wait_role(&api, &a, "fenced", Duration::from_secs(15)).await;
    assert_eq!(fenced["active"], false);
    sleep(Duration::from_secs(3)).await;
    let sb = api.status(&b).await.unwrap();
    assert_eq!(role_of(&sb), "standby", "no arbiter, no takeover: {sb}");
    assert_passive(&api, &a).await;
    let arbiter = start_arbiter(state.clone()).await;
    let back = wait_role(&api, &a, "primary", Duration::from_secs(40)).await;
    assert_eq!(back["epoch"], pa["epoch"], "the same term resumes");
    wait_streaming(&api, &b).await;

    // 3. The primary dies for real: once its lease ran out the standby takes over.
    let t = Instant::now();
    a.kill();
    let promoted = wait_role(&api, &b, "primary", Duration::from_secs(30)).await;
    measured(format!(
        "kill -9 primary with an arbiter: standby active after {} ms",
        t.elapsed().as_millis()
    ));
    assert!(promoted["epoch"].as_u64().unwrap() > pa["epoch"].as_u64().unwrap());

    // 4. The old primary returns: it asks the arbiter, is told the lease is B's, and follows.
    a.start();
    let first = wait_up(&a).await;
    assert_eq!(
        first.as_deref(),
        Some("standby"),
        "a restarted old primary must come up passive"
    );
    wait_streaming(&api, &a).await;

    let (violations, samples) = monitor.finish().await;
    measured(format!(
        "dual-active samples with an arbiter: {violations} of {samples}"
    ));
    assert!(samples > 100);
    assert_eq!(violations, 0, "two nodes reported active at the same time");
    arbiter.abort();
}

// ─────────────────── a node with HA off is the node it always was ───────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_node_with_ha_disabled_behaves_as_before() {
    let _serial = SERIAL.lock().await;
    let root = tempfile::tempdir().unwrap();
    let mut n = Node::new(root.path(), "solo", "primary", "127.0.0.1:1".into());
    n.ha_enabled = false;
    n.extra_yaml = "devices:\n  - id: \"boot_dev\"\n    ip: \"127.0.0.1\"\n    port: 1\n    unit_id: 1\n    interval_ms: 1000\n    endianness: \"big\"\n    registers:\n      - name: \"v\"\n        address: 0\n        type: \"float32\"\n".into();

    // The database was used by a pair before: triggers and a change log exist.
    seed_admin(&n.db()).await;
    {
        let c = Connection::open(n.db()).unwrap();
        voltrus_enterprise::ha::store::init_schema(&c).unwrap();
        voltrus_enterprise::ha::store::install_triggers(&c).unwrap();
        c.execute(
            "INSERT INTO settings (key, value) VALUES ('probe', '1')",
            [],
        )
        .unwrap();
        assert!(
            voltrus_enterprise::ha::store::max_seq(&c).unwrap() > 0,
            "the capture was active"
        );
    }

    n.start();
    wait_up(&n).await;
    let api = Api::login(&n).await;

    // The boot path started the pollers, exactly as the open core does.
    assert!(
        poller_ids(&api, &n).await.contains(&"boot_dev".to_string()),
        "boot starts the configured devices"
    );

    // HA off: no capture triggers, no change log, no guard. (`ha_node`, the node's
    // recorded role, stays: it is a single row and keeps the identity if HA is turned on again.)
    let ha_objects = scalar(
        &n.db(),
        "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('ha_changelog', 'ha_guard') OR name LIKE 'ha_trg_%'",
    );
    assert_eq!(
        ha_objects,
        Some(0),
        "the leftovers of an earlier pair are removed"
    );

    // Work as usual, then check nothing grows.
    let (s, _, role) = api
        .post(&n, "/api/v1/modbus-devices", device_json("extra"))
        .await
        .unwrap();
    assert!((200..300).contains(&s), "writes are accepted: {s}");
    assert_eq!(role, None, "no passive marker on a single node");
    let writer = Writer::start(n.db(), "solo");
    sleep(Duration::from_millis(500)).await;
    writer.stop();
    assert_eq!(
        scalar(
            &n.db(),
            "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('ha_changelog', 'ha_guard') OR name LIKE 'ha_trg_%'"
        ),
        Some(0),
        "no change log appears while the node works"
    );
    let (s, st) = api.get(&n, "/api/v1/ha/status").await.unwrap();
    assert_eq!(s, 200);
    assert_eq!(
        (
            st["enabled"].clone(),
            st["role"].clone(),
            st["active"].clone()
        ),
        (json!(false), json!("primary"), json!(true))
    );
    let (s, v) = api
        .post(&n, "/api/v1/ha/force-failover", json!({}))
        .await
        .map(|(s, v, _)| (s, v))
        .unwrap();
    assert_eq!(s, 400, "no HA, nothing to switch: {v}");
}

// ───────────────────────── the cost of capturing changes ─────────────────────────

/// `cargo test --features ha-test-node --test ha_pair -- --ignored --nocapture changelog_overhead`
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "benchmark"]
async fn changelog_overhead_on_telemetry_inserts() {
    let _serial = SERIAL.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("bench.db");
    let storage = voltrus::storage::Storage::new(db.to_str().unwrap()).unwrap();

    async fn run(
        storage: &voltrus::storage::Storage,
        batches: usize,
        metrics: usize,
        base_ts: i64,
    ) -> f64 {
        let t = Instant::now();
        for i in 0..batches {
            let rows: Vec<(String, i64, String, f64)> = (0..metrics)
                .map(|m| {
                    (
                        "dev".to_string(),
                        base_ts + i as i64,
                        format!("m{m}"),
                        i as f64,
                    )
                })
                .collect();
            storage.insert_batch(&rows).await.unwrap();
        }
        (batches * metrics) as f64 / t.elapsed().as_secs_f64()
    }

    for (batches, metrics, label) in [
        (20_000usize, 6usize, "1 device, 6 metrics per batch"),
        (2_000, 600, "100 devices, 600 rows per batch"),
    ] {
        let off = run(&storage, batches, metrics, 0).await;
        storage
            .with_conn(|c| {
                voltrus_enterprise::ha::store::init_schema(c)?;
                voltrus_enterprise::ha::store::install_triggers(c)?;
                Ok(())
            })
            .await
            .unwrap();
        let on = run(&storage, batches, metrics, 10_000_000).await;
        let on_pruned = {
            // The shipper acknowledges and prunes as it goes.
            let t = Instant::now();
            for i in 0..batches {
                let rows: Vec<(String, i64, String, f64)> = (0..metrics)
                    .map(|m| {
                        (
                            "dev".to_string(),
                            20_000_000 + i as i64,
                            format!("m{m}"),
                            i as f64,
                        )
                    })
                    .collect();
                storage.insert_batch(&rows).await.unwrap();
                if i % 200 == 0 {
                    storage
                        .with_conn(|c| {
                            let max = voltrus_enterprise::ha::store::max_seq(c)?;
                            voltrus_enterprise::ha::store::prune_log(c, max)?;
                            Ok(())
                        })
                        .await
                        .unwrap();
                }
            }
            (batches * metrics) as f64 / t.elapsed().as_secs_f64()
        };
        measured(format!(
            "telemetry inserts, {label}: {off:.0} rows/s without triggers, {on:.0} rows/s with the change log ({:.0}% of baseline), {on_pruned:.0} rows/s with the log pruned as it is acknowledged",
            on / off * 100.0
        ));
        storage
            .with_conn(|c| {
                voltrus_enterprise::ha::store::teardown(c)?;
                Ok(())
            })
            .await
            .unwrap();
    }
}
