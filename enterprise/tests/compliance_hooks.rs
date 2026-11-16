//! 21 CFR Part 11 hooks end to end, on one `voltrus-ha-test-node` process (the
//! real add-on with a stub Enterprise licence, hot standby off).
//!
//! Run:  cargo test --features ha-test-node --test compliance_hooks -- --nocapture

#![cfg(unix)]

use std::net::TcpListener as StdListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use rusqlite::Connection;
use serde_json::{Value, json};

const AUTH_SECRET: &str = "compliance-test-auth-secret-0123456789";
const ADMIN_PASSWORD: &str = "Admin-Pass-01";

struct Node {
    dir: PathBuf,
    http: u16,
    child: Option<Child>,
}

impl Node {
    fn db(&self) -> PathBuf {
        self.dir.join("data.db")
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.http)
    }

    fn start(&mut self) {
        let yaml = format!(
            "server_port: {}\nserver_host: \"127.0.0.1\"\ndb_path: \"{}\"\npersist_interval_secs: 1\ncookie_secure: false\nauth_secret: \"{AUTH_SECRET}\"\n",
            self.http,
            self.db().display()
        );
        std::fs::write(self.dir.join("config.yaml"), yaml).unwrap();
        let log = std::fs::File::create(self.dir.join("node.log")).unwrap();
        self.child = Some(
            Command::new(env!("CARGO_BIN_EXE_voltrus-ha-test-node"))
                .arg("config.yaml")
                .current_dir(&self.dir)
                .env("RUST_LOG", "info")
                .env("VOLTRUS_ACTIVATION_URL", "http://127.0.0.1:9")
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log))
                .spawn()
                .expect("start the test node"),
        );
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        if let Some(mut c) = self.child.take() {
            c.kill().ok();
            c.wait().ok();
        }
        if std::thread::panicking() {
            let log = std::fs::read_to_string(self.dir.join("node.log")).unwrap_or_default();
            let tail: Vec<&str> = log.lines().rev().take(40).collect();
            eprintln!(
                "---- node log ----\n{}",
                tail.into_iter().rev().collect::<Vec<_>>().join("\n")
            );
        }
    }
}

/// A signed-in browser: both session cookies, sent back by hand.
struct Session {
    http: reqwest::Client,
    token: String,
    refresh: String,
    login: Value,
}

fn cookie(resp: &reqwest::Response, name: &str) -> Option<String> {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|c| c.starts_with(&format!("{name}=")))
        .map(|c| c.split(';').next().unwrap().to_string())
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
}

async fn login(node: &Node, user: &str, password: &str) -> Session {
    let http = client();
    let resp = http
        .post(node.url("/api/v1/auth/login"))
        .json(&json!({"username": user, "password": password}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login {user}");
    let token = cookie(&resp, "voltrus_token").expect("access cookie");
    let refresh = cookie(&resp, "voltrus_refresh").expect("refresh cookie");
    let login = resp.json().await.unwrap();
    Session {
        http,
        token,
        refresh,
        login,
    }
}

impl Session {
    async fn send(
        &self,
        method: reqwest::Method,
        node: &Node,
        path: &str,
        body: Option<Value>,
    ) -> (u16, Value) {
        let mut req = self
            .http
            .request(method, node.url(path))
            .header("Cookie", format!("{}; {}", self.token, self.refresh));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn get(&self, node: &Node, path: &str) -> (u16, Value) {
        self.send(reqwest::Method::GET, node, path, None).await
    }

    async fn post(&self, node: &Node, path: &str, body: Value) -> (u16, Value) {
        self.send(reqwest::Method::POST, node, path, Some(body))
            .await
    }

    async fn put(&self, node: &Node, path: &str, body: Value) -> (u16, Value) {
        self.send(reqwest::Method::PUT, node, path, Some(body))
            .await
    }

    async fn bytes(&self, node: &Node, path: &str) -> (u16, Vec<u8>) {
        let resp = self
            .http
            .get(node.url(path))
            .header("Cookie", format!("{}; {}", self.token, self.refresh))
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.bytes().await.unwrap().to_vec())
    }

    async fn refresh(&self, node: &Node) -> (u16, Value) {
        let resp = self
            .http
            .post(node.url("/api/v1/auth/refresh"))
            .header("Cookie", &self.refresh)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }
}

fn exec(db: &Path, sql: &str) {
    let c = Connection::open(db).unwrap();
    c.busy_timeout(Duration::from_secs(5)).unwrap();
    c.execute_batch(sql).unwrap();
}

async fn start_node() -> (tempfile::TempDir, Node, Vec<i64>) {
    let tmp = tempfile::tempdir().unwrap();
    let http = StdListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut node = Node {
        dir: tmp.path().to_path_buf(),
        http,
        child: None,
    };
    let alarms = {
        let storage = voltrus::storage::Storage::new(node.db().to_str().unwrap()).unwrap();
        storage
            .create_user(
                "admin",
                &voltrus::api::auth::hash_password(ADMIN_PASSWORD),
                "",
                "admin",
            )
            .await
            .unwrap();
        let mut ids = Vec::new();
        for i in 0..3 {
            ids.push(
                storage
                    .create_alarm(
                        &format!("line1.temp{i}"),
                        "warning",
                        80.0,
                        70.0,
                        "High temperature",
                        0.0,
                    )
                    .await
                    .unwrap(),
            );
        }
        ids
    };
    exec(
        &node.db(),
        "UPDATE users SET must_change_password = 0 WHERE username = 'admin'",
    );
    node.start();
    let http = client();
    for _ in 0..300 {
        if let Ok(r) = http.get(node.url("/api/v1/health")).send().await
            && r.status().is_success()
        {
            return (tmp, node, alarms);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the node did not come up");
}

fn policy(signatures: Value) -> Value {
    json!({
        "auto_logout_minutes": 15,
        "password_min_length": 10,
        "password_require_uppercase": true,
        "password_require_number": true,
        "password_require_special": true,
        "password_rotation_days": 90,
        "password_history": 3,
        "session_max_hours": 8,
        "signatures": signatures,
    })
}

fn ack_signed() -> Value {
    json!({
        "signature": {"password": "Strong-Pass1", "meaning": "Reviewed", "reason": "High temperature checked on site"}
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn part_11_policy_is_enforced_through_the_core_hooks() {
    let (_tmp, node, alarms) = start_node().await;
    let admin = login(&node, "admin", ADMIN_PASSWORD).await;

    // Nothing is enforced before the settings are saved.
    let (s, cfg) = admin.get(&node, "/api/v1/compliance/config").await;
    assert_eq!(s, 200);
    assert_eq!(cfg["enforced"], false);
    assert_eq!(cfg["signatures"]["actions"]["alarm_ack"]["enabled"], false);
    assert_eq!(
        cfg["signatures"]["actions"]["alarm_ack"]["meaning"],
        "Reviewed"
    );

    let signatures = json!({
        "actions": {
            "tag_write": {"enabled": false, "meaning": "Performed"},
            "alarm_ack": {"enabled": true, "meaning": "Reviewed"},
            "alarm_shelve": {"enabled": false, "meaning": "Reviewed"},
            "recipe_load": {"enabled": false, "meaning": "Approved"},
            "mode_switch": {"enabled": false, "meaning": "Approved"}
        },
        "tag_scope": "all",
        "tag_ids": [],
        "unattended_exceptions": []
    });
    let (s, body) = admin
        .put(
            &node,
            "/api/v1/compliance/config",
            policy(signatures.clone()),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["config"]["enforced"], true);

    // ── password policy ──
    let (s, body) = admin
        .post(
            &node,
            "/api/v1/users",
            json!({"username": "op1", "password": "weakpassword", "role": "operator"}),
        )
        .await;
    assert_eq!(s, 400, "{body}");
    assert_eq!(
        body["error"],
        "Password must contain at least one uppercase letter"
    );
    let (s, body) = admin
        .post(
            &node,
            "/api/v1/users",
            json!({"username": "Line-Op-01", "password": "line-op-01", "role": "operator"}),
        )
        .await;
    assert_eq!(s, 400);
    assert_eq!(
        body["error"],
        "Password cannot be the same as the user name"
    );
    let (s, body) = admin
        .post(
            &node,
            "/api/v1/users",
            json!({"username": "op1", "password": "Strong-Pass1", "role": "operator"}),
        )
        .await;
    assert!(s == 200 || s == 201, "{s} {body}");
    let (s, _) = admin
        .put(
            &node,
            "/api/v1/compliance/signer-names/op1",
            json!({"full_name": "Olivia Operator"}),
        )
        .await;
    assert_eq!(s, 200);

    let op = login(&node, "op1", "Strong-Pass1").await;
    assert_eq!(op.login["session"]["idle_timeout_secs"], 900);
    assert_eq!(op.login["session"]["absolute_timeout_secs"], 28_800);
    let change = |old: &str, new: &str| json!({"old_password": old, "new_password": new});
    // The current password is history too.
    let (s, body) = op
        .post(
            &node,
            "/api/v1/auth/change-password",
            change("Strong-Pass1", "Strong-Pass1"),
        )
        .await;
    assert_eq!(s, 400, "{body}");
    assert!(body["error"].as_str().unwrap().contains("used recently"));
    let (s, body) = op
        .post(
            &node,
            "/api/v1/auth/change-password",
            change("Strong-Pass1", "Second-Pass2"),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    let (s, body) = op
        .post(
            &node,
            "/api/v1/auth/change-password",
            change("Second-Pass2", "Strong-Pass1"),
        )
        .await;
    assert_eq!(s, 400, "back to a recent password: {body}");
    let (s, body) = op
        .post(
            &node,
            "/api/v1/auth/change-password",
            change("Second-Pass2", "Third-Pass-33"),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    let (s, body) = op
        .post(
            &node,
            "/api/v1/auth/change-password",
            change("Third-Pass-33", "Strong-Pass1"),
        )
        .await;
    assert_eq!(s, 400, "still within the last 3: {body}");

    // Only hashes are stored.
    let c = Connection::open(node.db()).unwrap();
    let stored: Vec<String> = c
        .prepare("SELECT password_hash FROM compliance_password_history WHERE username = 'op1' AND password_hash != ''")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert!(stored.len() >= 3);
    assert!(stored.iter().all(|h| h.starts_with("$argon2")));

    // ── expiry at sign-in ──
    let fresh = login(&node, "op1", "Third-Pass-33").await;
    assert_eq!(fresh.login["must_change_password"], false);
    exec(
        &node.db(),
        "UPDATE compliance_password_history SET changed_at = changed_at - 91 * 86400 WHERE username = 'op1'",
    );
    let history_rows = |c: &Connection| -> i64 {
        c.query_row(
            "SELECT COUNT(*) FROM compliance_password_history WHERE username = 'op1'",
            [],
            |r| r.get(0),
        )
        .unwrap()
    };
    let must_change = |c: &Connection| -> bool {
        c.query_row(
            "SELECT must_change_password FROM users WHERE username = 'op1'",
            [],
            |r| r.get(0),
        )
        .unwrap()
    };
    let rows_before = history_rows(&c);
    let expired = login(&node, "op1", "Third-Pass-33").await;
    assert_eq!(expired.login["must_change_password"], true);
    assert_eq!(expired.login["password_expired"], true);
    // The core keeps the flag on the account: the session opened before the
    // expiry is held to the change too.
    assert!(must_change(&c));
    let (s, body) = fresh.get(&node, "/api/v1/screens").await;
    assert_eq!(s, 403, "{body}");
    assert_eq!(body["code"], "password_change_required");
    let (s, me) = expired.get(&node, "/api/v1/auth/me").await;
    assert_eq!(s, 200);
    assert_eq!(me["must_change_password"], true);
    assert_eq!(me["password_expired"], true);
    // Signing in again while flagged answers the same and records nothing.
    let again = login(&node, "op1", "Third-Pass-33").await;
    assert_eq!(again.login["must_change_password"], true);
    assert_eq!(again.login["password_expired"], true);
    assert_eq!(history_rows(&c), rows_before, "no row per sign-in or /me");
    let (s, _) = expired
        .post(
            &node,
            "/api/v1/auth/change-password",
            change("Third-Pass-33", "Fourth-Pass-44"),
        )
        .await;
    assert_eq!(s, 200);
    // The change clears the flag and restarts the clock from the new hash.
    assert!(!must_change(&c));
    let (kind, hash, age): (String, String, i64) = c
        .query_row(
            "SELECT kind, password_hash, strftime('%s','now') - changed_at FROM compliance_password_history
             WHERE username = 'op1' ORDER BY changed_at DESC, id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(kind, "self_service");
    assert!(hash.starts_with("$argon2"));
    assert!(age < 60, "changed {age}s ago");
    let (s, body) = again.get(&node, "/api/v1/screens").await;
    assert_eq!(s, 200, "released once changed: {body}");
    let renewed = login(&node, "op1", "Fourth-Pass-44").await;
    assert_eq!(renewed.login["must_change_password"], false);
    assert_eq!(renewed.login["password_expired"], false);
    let (_, me) = renewed.get(&node, "/api/v1/auth/me").await;
    assert_eq!(me["must_change_password"], false);
    assert!(!must_change(&c));

    // An account the add-on never saw set (made before the policy): its first
    // sign-in starts the clock once, and later sign-ins neither expire it nor
    // add rows.
    exec(
        &node.db(),
        "DELETE FROM compliance_password_history WHERE username = 'admin'",
    );
    let first = login(&node, "admin", ADMIN_PASSWORD).await;
    assert_eq!(first.login["must_change_password"], false);
    assert_eq!(first.login["password_expired"], false);
    let second = login(&node, "admin", ADMIN_PASSWORD).await;
    assert_eq!(second.login["must_change_password"], false);
    let baselines: Vec<String> = c
        .prepare("SELECT kind FROM compliance_password_history WHERE username = 'admin'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(baselines, vec!["baseline".to_string()]);
    let (s, body) = second.get(&node, "/api/v1/screens").await;
    assert_eq!(s, 200, "{body}");
    let (s, _) = admin
        .put(
            &node,
            "/api/v1/users/2",
            json!({"password": "Strong-Pass1"}),
        )
        .await;
    // An admin reset follows the same rules (op1 is user 2) and resets the clock.
    assert_eq!(s, 200);

    // ── idle and absolute timeout ──
    let idle = login(&node, "op1", "Strong-Pass1").await;
    let (s, _) = idle.refresh(&node).await;
    assert_eq!(s, 200, "a fresh session refreshes");
    let idle = login(&node, "op1", "Strong-Pass1").await;
    exec(
        &node.db(),
        "UPDATE refresh_tokens SET last_activity_at = strftime('%s','now') - 3600 WHERE username = 'op1' AND revoked_at IS NULL",
    );
    let (s, body) = idle.refresh(&node).await;
    assert_eq!(s, 401, "{body}");
    assert_eq!(body["code"], "session_idle");

    let long = login(&node, "op1", "Strong-Pass1").await;
    exec(
        &node.db(),
        "UPDATE refresh_tokens SET session_started_at = strftime('%s','now') - 9 * 3600, last_activity_at = strftime('%s','now') WHERE username = 'op1' AND revoked_at IS NULL",
    );
    let (s, body) = long.refresh(&node).await;
    assert_eq!(s, 401, "{body}");
    assert_eq!(body["code"], "session_expired");

    // ── a signed alarm acknowledgement ──
    let op = login(&node, "op1", "Strong-Pass1").await;
    let ack = format!("/api/v1/alarms/{}/acknowledge", alarms[0]);
    let (s, body) = op.post(&node, &ack, json!({})).await;
    assert_eq!(s, 428, "{body}");
    assert_eq!(body["signature_required"]["action"], "alarm_ack");
    assert_eq!(body["signature_required"]["meaning"], "Reviewed");
    let (s, _) = op
        .post(
            &node,
            &ack,
            json!({"signature": {"password": "Strong-Pass1", "meaning": "Approved", "reason": "x"}}),
        )
        .await;
    assert_eq!(s, 428, "another meaning is not this signature");
    let (s, body) = op.post(&node, &ack, ack_signed()).await;
    assert_eq!(s, 200, "{body}");

    // The record is in the e-signature records, linked to the core audit entry.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let (s, csv) = admin
        .bytes(&node, "/api/v1/compliance/signatures/export")
        .await;
    assert_eq!(s, 200);
    let csv = String::from_utf8(csv).unwrap();
    let row = csv
        .lines()
        .find(|l| l.contains(",alarm_ack,"))
        .unwrap_or_else(|| panic!("no alarm_ack row in\n{csv}"));
    assert!(row.contains(",op1,Reviewed,High temperature checked on site,alarm_ack,alarm,"));
    assert!(row.contains(",true,Olivia Operator,session,"), "{row}");
    let audit_id: i64 = row.rsplit(',').next().unwrap().parse().expect(row);
    let sig_id: i64 = row.split(',').next().unwrap().parse().unwrap();
    let details: String = c
        .query_row(
            "SELECT details FROM audit_log WHERE id = ?1 AND action = 'signature'",
            [audit_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        details.starts_with(&format!("action=alarm_ack signature={sig_id} ")),
        "{details}"
    );

    let (s, pdf) = admin
        .bytes(&node, "/api/v1/compliance/signatures/export.pdf")
        .await;
    assert_eq!(s, 200);
    let pdf = String::from_utf8_lossy(&pdf);
    for want in [
        "Olivia Operator \\(op1\\)",
        "alarm_ack \\(signed in a session as operator\\)",
        "High temperature checked on site",
        &format!("#{audit_id} \\(action signature\\)"),
    ] {
        assert!(pdf.contains(want), "PDF misses {want}");
    }
    let (s, v) = admin
        .get(&node, "/api/v1/compliance/signatures/verify")
        .await;
    assert_eq!(s, 200);
    assert_eq!(v["ok"], true, "{v}");

    // ── unattended callers ──
    let (s, key) = admin
        .post(
            &node,
            "/api/v1/api-keys",
            json!({"name": "mes", "role": "integration"}),
        )
        .await;
    assert_eq!(s, 200, "{key}");
    let key_id = key["id"].as_i64().unwrap();
    let plain = key["key"].as_str().unwrap().to_string();
    let by_key = |alarm: i64| {
        let http = client();
        let url = node.url(&format!("/api/v1/alarms/{alarm}/acknowledge"));
        let plain = plain.clone();
        async move {
            let r = http
                .post(url)
                .header("X-API-Key", plain)
                .send()
                .await
                .unwrap();
            let s = r.status().as_u16();
            (s, r.json::<Value>().await.unwrap_or(Value::Null))
        }
    };
    let (s, body) = by_key(alarms[1]).await;
    assert_eq!(s, 403, "{body}");
    assert_eq!(body["code"], "signature_not_possible");

    let mut with_exception = signatures.clone();
    with_exception["unattended_exceptions"] = json!([
        {"origin": "api_key", "principal": format!("api_key:{key_id}"), "note": "MES line 1, validated"}
    ]);
    let (s, body) = admin
        .put(&node, "/api/v1/compliance/config", policy(with_exception))
        .await;
    assert_eq!(s, 200, "{body}");
    let (s, body) = by_key(alarms[1]).await;
    assert_eq!(s, 200, "the named key may acknowledge unsigned: {body}");

    // Turning the requirement off lets a person acknowledge without signing.
    let mut off = signatures.clone();
    off["actions"]["alarm_ack"]["enabled"] = json!(false);
    let (s, _) = admin
        .put(&node, "/api/v1/compliance/config", policy(off))
        .await;
    assert_eq!(s, 200);
    let (s, body) = op
        .post(
            &node,
            &format!("/api/v1/alarms/{}/acknowledge", alarms[2]),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{body}");
}
