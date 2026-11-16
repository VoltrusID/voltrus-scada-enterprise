//! The Part 11 settings the add-on enforces through the core's edition hooks.
//!
//! The password rules and the idle timeout are the core's `ComplianceConfig`
//! (stored in the core `compliance_config` table, as before). What the core
//! struct has no field for lives here, in `compliance_policy`: password history,
//! the absolute session lifetime and which actions need an electronic signature.
//!
//! Nothing is enforced until an administrator has saved the Compliance settings
//! once (and the licence includes Part 11): a site that never opened the screen
//! keeps the core's behaviour.

use std::sync::RwLock;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use voltrus::edition::{ActionContext, ActionOrigin, SignedAction};
use voltrus::storage::Storage;

use super::ComplianceConfig;

/// Longest meaning text, as shown in the signing dialog and the records.
pub const MAX_MEANING_CHARS: usize = 80;
/// Most password hashes kept per user.
pub const MAX_PASSWORD_HISTORY: u32 = 24;
/// Most unattended exceptions.
pub const MAX_EXCEPTIONS: usize = 100;
/// How long a read policy is reused before it is read again. Short, so the
/// standby of a hot standby pair follows a change made on the active node.
const CACHE_TTL: Duration = Duration::from_secs(5);

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS compliance_policy (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        config TEXT NOT NULL,
        updated_at INTEGER NOT NULL,
        updated_by TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS compliance_password_history (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL,
        password_hash TEXT NOT NULL,
        changed_at INTEGER NOT NULL,
        kind TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_compliance_password_history_user
        ON compliance_password_history(username, changed_at);
    CREATE TABLE IF NOT EXISTS compliance_signature_context (
        signature_id INTEGER PRIMARY KEY,
        origin TEXT NOT NULL,
        role TEXT NOT NULL,
        details TEXT NOT NULL,
        checksum TEXT NOT NULL
    );";

pub(crate) fn ensure_schema(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)
}

/// One signable action: whether it needs a signature, and what signing means.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionRule {
    #[serde(default)]
    pub enabled: bool,
    pub meaning: String,
}

impl ActionRule {
    fn off(meaning: &str) -> Self {
        Self {
            enabled: false,
            meaning: meaning.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SignedActions {
    pub tag_write: ActionRule,
    pub alarm_ack: ActionRule,
    pub alarm_shelve: ActionRule,
    pub recipe_load: ActionRule,
    pub mode_switch: ActionRule,
}

impl Default for SignedActions {
    fn default() -> Self {
        Self {
            tag_write: ActionRule::off("Performed"),
            alarm_ack: ActionRule::off("Reviewed"),
            alarm_shelve: ActionRule::off("Reviewed"),
            recipe_load: ActionRule::off("Approved"),
            mode_switch: ActionRule::off("Approved"),
        }
    }
}

impl SignedActions {
    pub fn rule(&self, action: SignedAction) -> &ActionRule {
        match action {
            SignedAction::TagWrite => &self.tag_write,
            SignedAction::AlarmAck => &self.alarm_ack,
            SignedAction::AlarmShelve => &self.alarm_shelve,
            SignedAction::RecipeLoad => &self.recipe_load,
            SignedAction::ModeSwitch => &self.mode_switch,
        }
    }

    fn all(&self) -> [(&'static str, &ActionRule); 5] {
        [
            ("tag_write", &self.tag_write),
            ("alarm_ack", &self.alarm_ack),
            ("alarm_shelve", &self.alarm_shelve),
            ("recipe_load", &self.recipe_load),
            ("mode_switch", &self.mode_switch),
        ]
    }
}

/// Which tag writes need a signature when tag writes are signed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagScope {
    /// Every tag write.
    #[default]
    All,
    /// Tags whose write policy has "requires a reason" set (the core's only
    /// per-tag write policy flag).
    RequiresReason,
    /// The tags listed in `tag_ids`.
    Listed,
}

/// Unattended callers that may act unsigned although the action needs a
/// signature. Without an exception they are refused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnattendedException {
    /// `api_key`, `mcp` or `flow`.
    pub origin: String,
    /// The caller as the core names it: `api_key:<id>`, `flow:<id>`, or for MCP
    /// the user or `api_key:<id>` it acts for. `*` is every caller of that origin.
    pub principal: String,
    /// Why it is allowed (the validation that covers it). Shown in the settings.
    #[serde(default)]
    pub note: String,
}

impl UnattendedException {
    fn matches(&self, ctx: &ActionContext) -> bool {
        origin_name(ctx.origin) == Some(self.origin.as_str())
            && (self.principal == "*" || self.principal == ctx.username)
    }
}

fn origin_name(origin: ActionOrigin) -> Option<&'static str> {
    match origin {
        ActionOrigin::ApiKey => Some("api_key"),
        ActionOrigin::Mcp => Some("mcp"),
        ActionOrigin::Flow => Some("flow"),
        ActionOrigin::Session | ActionOrigin::Recipe => None,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignaturePolicy {
    #[serde(default)]
    pub actions: SignedActions,
    #[serde(default)]
    pub tag_scope: TagScope,
    #[serde(default)]
    pub tag_ids: Vec<i64>,
    #[serde(default)]
    pub unattended_exceptions: Vec<UnattendedException>,
}

/// The settings kept in `compliance_policy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyExtras {
    /// Refuse a new password equal to one of the last N (0 = no history).
    #[serde(default = "default_history")]
    pub password_history: u32,
    /// Sign a session out this many hours after sign-in, active or not (0 = no limit).
    #[serde(default = "default_session_hours")]
    pub session_max_hours: u32,
    #[serde(default)]
    pub signatures: SignaturePolicy,
}

fn default_history() -> u32 {
    5
}

fn default_session_hours() -> u32 {
    12
}

impl Default for PolicyExtras {
    fn default() -> Self {
        Self {
            password_history: default_history(),
            session_max_hours: default_session_hours(),
            signatures: SignaturePolicy::default(),
        }
    }
}

/// Everything the hooks need, read together.
#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// An administrator saved the Compliance settings: enforce them.
    pub saved: bool,
    pub config: ComplianceConfig,
    pub extras: PolicyExtras,
}

/// Check the extras an administrator sends. Returns the cleaned value.
pub fn validate_extras(mut extras: PolicyExtras) -> Result<PolicyExtras, String> {
    if extras.password_history > MAX_PASSWORD_HISTORY {
        return Err(format!(
            "password_history is at most {MAX_PASSWORD_HISTORY}"
        ));
    }
    if extras.session_max_hours > 24 * 31 {
        return Err("session_max_hours is at most 744 (31 days)".into());
    }
    let sig = &mut extras.signatures;
    for rule in [
        &mut sig.actions.tag_write,
        &mut sig.actions.alarm_ack,
        &mut sig.actions.alarm_shelve,
        &mut sig.actions.recipe_load,
        &mut sig.actions.mode_switch,
    ] {
        rule.meaning = rule
            .meaning
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
    }
    for (name, rule) in sig.actions.all() {
        if rule.meaning.is_empty() {
            return Err(format!("the meaning of {name} cannot be empty"));
        }
        if rule.meaning.chars().count() > MAX_MEANING_CHARS {
            return Err(format!(
                "the meaning of {name} has at most {MAX_MEANING_CHARS} characters"
            ));
        }
        if rule.meaning.chars().any(|c| c.is_control()) {
            return Err(format!(
                "the meaning of {name} cannot contain control characters"
            ));
        }
    }
    sig.tag_ids.sort_unstable();
    sig.tag_ids.dedup();
    if sig.tag_scope == TagScope::Listed && sig.tag_ids.is_empty() && sig.actions.tag_write.enabled
    {
        return Err("choose at least one tag, or sign every tag write".into());
    }
    if sig.unattended_exceptions.len() > MAX_EXCEPTIONS {
        return Err(format!("at most {MAX_EXCEPTIONS} unattended exceptions"));
    }
    for e in &mut sig.unattended_exceptions {
        e.principal = e.principal.trim().to_string();
        e.note = e.note.trim().chars().take(200).collect();
        if !matches!(e.origin.as_str(), "api_key" | "mcp" | "flow") {
            return Err(format!(
                "unattended exception origin \"{}\" must be api_key, mcp or flow",
                e.origin
            ));
        }
        if e.principal.is_empty() || e.principal.chars().any(|c| c.is_whitespace()) {
            return Err(
                "an unattended exception names one caller (api_key:<id>, flow:<id>, a user name) or *"
                    .into(),
            );
        }
        let prefix_ok = match e.origin.as_str() {
            "api_key" => e.principal == "*" || e.principal.starts_with("api_key:"),
            "flow" => e.principal == "*" || e.principal.starts_with("flow:"),
            _ => true,
        };
        if !prefix_ok {
            return Err(format!(
                "a {} exception names {}:<id> or *",
                e.origin, e.origin
            ));
        }
    }
    Ok(extras)
}

/// Whether the policy asks for a signature for `ctx` (before the tag scope,
/// which needs the tag's write policy). Returns the meaning to sign.
pub fn requirement_for<'a>(
    policy: &'a Policy,
    ctx: &ActionContext,
    tag_in_scope: impl FnOnce(&SignaturePolicy) -> bool,
) -> Option<&'a str> {
    if !policy.saved {
        return None;
    }
    let sig = &policy.extras.signatures;
    let rule = sig.actions.rule(ctx.action);
    if !rule.enabled {
        return None;
    }
    if ctx.action == SignedAction::TagWrite && !tag_in_scope(sig) {
        return None;
    }
    // Callers that cannot sign are refused (the core answers 403), unless the
    // administrator made a named exception for them.
    if !ctx.origin.can_sign() && sig.unattended_exceptions.iter().any(|e| e.matches(ctx)) {
        return None;
    }
    Some(rule.meaning.as_str())
}

/// The tag id of a tag write's resource (`data_source:<id>`).
pub fn tag_id_of(resource: &str) -> Option<i64> {
    resource.strip_prefix("data_source:")?.parse().ok()
}

// ── storage ──

static CACHE: RwLock<Option<(Instant, Policy)>> = RwLock::new(None);

/// Forget the cached policy (after a save).
pub fn invalidate() {
    *CACHE.write().unwrap_or_else(|e| e.into_inner()) = None;
}

/// The policy, from a cache at most a few seconds old.
pub async fn current(storage: &Storage) -> Policy {
    if let Some((at, p)) = CACHE.read().unwrap_or_else(|e| e.into_inner()).as_ref()
        && at.elapsed() < CACHE_TTL
    {
        return p.clone();
    }
    match load(storage).await {
        Ok(p) => {
            *CACHE.write().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), p.clone()));
            p
        }
        Err(e) => {
            // Failing open here would silently stop enforcing the policy; the
            // last known one is the safer answer, the defaults otherwise.
            tracing::error!("compliance policy could not be read: {e:#}");
            CACHE
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
                .map(|(_, p)| p.clone())
                .unwrap_or_default()
        }
    }
}

pub async fn load(storage: &Storage) -> anyhow::Result<Policy> {
    let config = storage.get_compliance_config().await?;
    let extras = storage
        .with_conn(|c| {
            ensure_schema(c)?;
            let mut stmt = c.prepare("SELECT config FROM compliance_policy WHERE id = 1")?;
            let mut rows = stmt.query([])?;
            Ok(match rows.next()? {
                Some(r) => Some(r.get::<_, String>(0)?),
                None => None,
            })
        })
        .await?;
    let extras = match extras {
        Some(json) => serde_json::from_str(&json)?,
        None => PolicyExtras::default(),
    };
    Ok(Policy {
        saved: config.is_some(),
        config: config.unwrap_or_default(),
        extras,
    })
}

pub async fn save_extras(storage: &Storage, extras: &PolicyExtras, by: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string(extras)?;
    let by = by.to_string();
    storage
        .with_conn(move |c| {
            ensure_schema(c)?;
            c.execute(
                "INSERT INTO compliance_policy (id, config, updated_at, updated_by) VALUES (1, ?1, ?2, ?3)
                 ON CONFLICT(id) DO UPDATE SET config = excluded.config, updated_at = excluded.updated_at, updated_by = excluded.updated_by",
                rusqlite::params![json, voltrus::timeutil::now_secs(), by],
            )?;
            Ok(())
        })
        .await?;
    invalidate();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx(
        action: SignedAction,
        origin: ActionOrigin,
        user: &str,
        resource: &str,
    ) -> ActionContext {
        ActionContext {
            action,
            origin,
            username: user.into(),
            role: "operator".into(),
            resource: resource.into(),
            details: json!({}),
        }
    }

    fn policy(f: impl FnOnce(&mut SignaturePolicy)) -> Policy {
        let mut p = Policy {
            saved: true,
            ..Default::default()
        };
        f(&mut p.extras.signatures);
        p
    }

    #[test]
    fn nothing_is_required_by_default_or_before_the_settings_are_saved() {
        let p = Policy {
            saved: true,
            ..Default::default()
        };
        for a in [
            SignedAction::TagWrite,
            SignedAction::AlarmAck,
            SignedAction::AlarmShelve,
            SignedAction::RecipeLoad,
            SignedAction::ModeSwitch,
        ] {
            let c = ctx(a, ActionOrigin::Session, "op", "alarm:1");
            assert_eq!(requirement_for(&p, &c, |_| true), None);
        }
        let mut unsaved = policy(|s| s.actions.alarm_ack.enabled = true);
        unsaved.saved = false;
        let c = ctx(
            SignedAction::AlarmAck,
            ActionOrigin::Session,
            "op",
            "alarm:1",
        );
        assert_eq!(requirement_for(&unsaved, &c, |_| true), None);
    }

    #[test]
    fn each_action_carries_its_own_meaning() {
        let p = policy(|s| {
            s.actions.alarm_ack.enabled = true;
            s.actions.recipe_load.enabled = true;
            s.actions.recipe_load.meaning = "Approved for batch".into();
        });
        let ack = ctx(
            SignedAction::AlarmAck,
            ActionOrigin::Session,
            "op",
            "alarm:1",
        );
        assert_eq!(requirement_for(&p, &ack, |_| true), Some("Reviewed"));
        let load = ctx(
            SignedAction::RecipeLoad,
            ActionOrigin::Session,
            "op",
            "recipe:3",
        );
        assert_eq!(
            requirement_for(&p, &load, |_| true),
            Some("Approved for batch")
        );
        let shelve = ctx(
            SignedAction::AlarmShelve,
            ActionOrigin::Session,
            "op",
            "alarm:1",
        );
        assert_eq!(requirement_for(&p, &shelve, |_| true), None);
    }

    #[test]
    fn tag_writes_follow_the_scope() {
        let p = policy(|s| s.actions.tag_write.enabled = true);
        let w = ctx(
            SignedAction::TagWrite,
            ActionOrigin::Session,
            "op",
            "data_source:7",
        );
        assert_eq!(requirement_for(&p, &w, |_| true), Some("Performed"));
        assert_eq!(requirement_for(&p, &w, |_| false), None);
        // The scope is not consulted for other actions.
        let p = policy(|s| s.actions.mode_switch.enabled = true);
        let m = ctx(
            SignedAction::ModeSwitch,
            ActionOrigin::Session,
            "op",
            "data_source:7:write_policy",
        );
        assert_eq!(requirement_for(&p, &m, |_| false), Some("Approved"));
        assert_eq!(tag_id_of("data_source:7"), Some(7));
        assert_eq!(tag_id_of("data_source:7:write_policy"), None);
        assert_eq!(tag_id_of("alarm:7"), None);
    }

    #[test]
    fn unattended_callers_are_refused_unless_named() {
        let p = policy(|s| {
            s.actions.tag_write.enabled = true;
            s.unattended_exceptions = vec![
                UnattendedException {
                    origin: "flow".into(),
                    principal: "flow:4".into(),
                    note: "validated CIP flow".into(),
                },
                UnattendedException {
                    origin: "api_key".into(),
                    principal: "api_key:2".into(),
                    note: String::new(),
                },
            ];
        });
        let r = |origin, user: &str| {
            requirement_for(
                &p,
                &ctx(SignedAction::TagWrite, origin, user, "data_source:1"),
                |_| true,
            )
        };
        assert_eq!(r(ActionOrigin::Flow, "flow:4"), None);
        assert_eq!(r(ActionOrigin::Flow, "flow:5"), Some("Performed"));
        assert_eq!(r(ActionOrigin::ApiKey, "api_key:2"), None);
        assert_eq!(r(ActionOrigin::ApiKey, "api_key:3"), Some("Performed"));
        // An MCP call by the same key is a different origin: still refused.
        assert_eq!(r(ActionOrigin::Mcp, "api_key:2"), Some("Performed"));
        // A person is always asked to sign, exceptions or not.
        assert_eq!(r(ActionOrigin::Session, "flow:4"), Some("Performed"));
        assert_eq!(r(ActionOrigin::Recipe, "op"), Some("Performed"));
    }

    #[test]
    fn a_wildcard_exception_covers_one_origin() {
        let p = policy(|s| {
            s.actions.alarm_ack.enabled = true;
            s.unattended_exceptions = vec![UnattendedException {
                origin: "mcp".into(),
                principal: "*".into(),
                note: String::new(),
            }];
        });
        let c = |origin, user: &str| {
            requirement_for(
                &p,
                &ctx(SignedAction::AlarmAck, origin, user, "alarm:1"),
                |_| true,
            )
        };
        assert_eq!(c(ActionOrigin::Mcp, "anyone"), None);
        assert_eq!(c(ActionOrigin::ApiKey, "api_key:1"), Some("Reviewed"));
    }

    #[test]
    fn extras_are_validated() {
        let ok = validate_extras(PolicyExtras::default()).unwrap();
        assert_eq!(ok.signatures.actions.alarm_ack.meaning, "Reviewed");

        let mut e = PolicyExtras::default();
        e.signatures.actions.alarm_ack.meaning = "  Reviewed   by QA ".into();
        assert_eq!(
            validate_extras(e)
                .unwrap()
                .signatures
                .actions
                .alarm_ack
                .meaning,
            "Reviewed by QA"
        );

        let mut e = PolicyExtras::default();
        e.signatures.actions.tag_write.meaning = " ".into();
        assert!(validate_extras(e).is_err());

        let e = PolicyExtras {
            password_history: MAX_PASSWORD_HISTORY + 1,
            ..Default::default()
        };
        assert!(validate_extras(e).is_err());

        let mut e = PolicyExtras::default();
        e.signatures.actions.tag_write.enabled = true;
        e.signatures.tag_scope = TagScope::Listed;
        assert!(validate_extras(e).is_err());

        let mut e = PolicyExtras::default();
        e.signatures.unattended_exceptions = vec![UnattendedException {
            origin: "flow".into(),
            principal: "api_key:1".into(),
            note: String::new(),
        }];
        assert!(validate_extras(e).is_err());

        let mut e = PolicyExtras::default();
        e.signatures.unattended_exceptions = vec![UnattendedException {
            origin: "session".into(),
            principal: "*".into(),
            note: String::new(),
        }];
        assert!(validate_extras(e).is_err());
    }

    #[test]
    fn stored_extras_without_new_fields_still_parse() {
        let e: PolicyExtras = serde_json::from_str("{}").unwrap();
        assert_eq!(e, PolicyExtras::default());
        let e: PolicyExtras =
            serde_json::from_str(r#"{"signatures":{"actions":{"tag_write":{"enabled":true,"meaning":"Done"},"alarm_ack":{"meaning":"R"},"alarm_shelve":{"meaning":"R"},"recipe_load":{"meaning":"A"},"mode_switch":{"meaning":"A"}}}}"#)
                .unwrap();
        assert!(e.signatures.actions.tag_write.enabled);
        assert_eq!(e.signatures.tag_scope, TagScope::All);
    }
}
