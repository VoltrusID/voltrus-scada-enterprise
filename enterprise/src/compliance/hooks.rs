//! The core's compliance hooks (`voltrus::edition::Extension`), answered from
//! the Part 11 settings (`policy`). See core `docs/edition-hooks.md` for when
//! the core calls each one, and docs/enterprise/part-11.md for what a site sees.

use voltrus::edition::{
    ActionContext, PasswordChangeKind, PasswordCheck, SessionPolicy, SignatureRecord,
    SignatureRequirement,
};
use voltrus::models::User;
use voltrus::storage::Storage;

use super::policy::{self, Policy, TagScope};

/// The storage and the current policy, when the licence includes Part 11.
async fn licensed() -> Option<(&'static Storage, Policy)> {
    let rt = crate::try_rt()?;
    if !rt.tier().cfr_part_11() {
        return None;
    }
    let p = policy::current(&rt.storage).await;
    Some((&rt.storage, p))
}

/// The policy in force: licensed and saved by an administrator.
async fn enforced() -> Option<(&'static Storage, Policy)> {
    licensed().await.filter(|(_, p)| p.saved)
}

// ── passwords ──

/// The rules that need no stored data: length, character classes, user name.
pub fn check_password_rules(check: &PasswordCheck, p: &Policy) -> Result<(), String> {
    if check
        .new_password
        .trim()
        .eq_ignore_ascii_case(check.username.trim())
    {
        return Err("Password cannot be the same as the user name".into());
    }
    super::verify_password_complexity(&check.new_password, &p.config)
}

/// The last `n` stored password hashes of `username`, newest first, with the
/// account's current hash first (users created before the add-on have no
/// history row yet).
async fn recent_hashes(storage: &Storage, username: &str, n: u32) -> anyhow::Result<Vec<String>> {
    let mut hashes = Vec::new();
    if let Some(u) = storage.get_user_by_username(username).await?
        && !u.password_hash.is_empty()
    {
        hashes.push(u.password_hash);
    }
    let user = username.to_string();
    let stored = storage
        .with_conn(move |c| {
            policy::ensure_schema(c)?;
            let mut stmt = c.prepare(
                "SELECT password_hash FROM compliance_password_history
                 WHERE username = ?1 AND password_hash != ''
                 ORDER BY changed_at DESC, id DESC LIMIT ?2",
            )?;
            let rows = stmt
                .query_map(rusqlite::params![user, n], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
        .await?;
    for h in stored {
        if !hashes.contains(&h) {
            hashes.push(h);
        }
    }
    hashes.truncate(n as usize);
    Ok(hashes)
}

pub async fn validate_new_password(check: &PasswordCheck) -> Result<(), String> {
    let Some((storage, p)) = enforced().await else {
        return Ok(());
    };
    check_password_rules(check, &p)?;
    let n = p.extras.password_history;
    if n == 0 || check.kind == PasswordChangeKind::AdminCreate {
        return Ok(());
    }
    let hashes = recent_hashes(storage, &check.username, n)
        .await
        .map_err(|e| {
            tracing::error!("password history of {}: {e:#}", check.username);
            "The password history could not be read; try again".to_string()
        })?;
    let candidate = check.new_password.clone();
    let reused = tokio::task::spawn_blocking(move || {
        hashes
            .iter()
            .any(|h| voltrus::api::auth::verify_password(&candidate, h))
    })
    .await
    .unwrap_or(false);
    if reused {
        return Err(format!(
            "Password was used recently: choose one that is not among your last {n}"
        ));
    }
    Ok(())
}

fn kind_name(kind: PasswordChangeKind) -> &'static str {
    match kind {
        PasswordChangeKind::SelfService => "self_service",
        PasswordChangeKind::AdminCreate => "admin_create",
        PasswordChangeKind::AdminReset => "admin_reset",
    }
}

pub async fn password_changed(username: &str, password_hash: &str, kind: PasswordChangeKind) {
    // Kept whenever Part 11 is licensed, so history and expiry are right from
    // the moment an administrator saves the policy.
    let Some((storage, _)) = licensed().await else {
        return;
    };
    let (user, hash) = (username.to_string(), password_hash.to_string());
    let res = storage
        .with_conn(move |c| {
            policy::ensure_schema(c)?;
            let tx = c.transaction()?;
            tx.execute(
                "INSERT INTO compliance_password_history (username, password_hash, changed_at, kind) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![user, hash, voltrus::timeutil::now_secs(), kind_name(kind)],
            )?;
            // Keep only as many hashes as the deepest history allowed.
            tx.execute(
                "DELETE FROM compliance_password_history WHERE username = ?1 AND id NOT IN (
                     SELECT id FROM compliance_password_history WHERE username = ?1
                     ORDER BY changed_at DESC, id DESC LIMIT ?2)",
                rusqlite::params![user, policy::MAX_PASSWORD_HISTORY],
            )?;
            tx.commit()?;
            Ok(())
        })
        .await;
    if let Err(e) = res {
        tracing::error!("password history of {username} not recorded: {e:#}");
    }
}

/// When `username`'s password was last set, or `None` when the add-on never saw
/// it set. In that case `now` is recorded as the start, so accounts that existed
/// before the policy get a full rotation period instead of expiring at once.
async fn last_changed(storage: &Storage, username: &str, now: i64) -> anyhow::Result<i64> {
    let user = username.to_string();
    storage
        .with_conn(move |c| {
            policy::ensure_schema(c)?;
            let last: Option<i64> = c.query_row(
                "SELECT MAX(changed_at) FROM compliance_password_history WHERE username = ?1",
                [&user],
                |r| r.get(0),
            )?;
            if let Some(t) = last {
                return Ok(t);
            }
            c.execute(
                "INSERT INTO compliance_password_history (username, password_hash, changed_at, kind) VALUES (?1, '', ?2, 'baseline')",
                rusqlite::params![user, now],
            )?;
            Ok(now)
        })
        .await
}

pub fn is_expired(last_changed: i64, now: i64, rotation_days: u32) -> bool {
    rotation_days > 0 && now - last_changed >= i64::from(rotation_days) * 86_400
}

pub async fn password_expired(user: &User) -> bool {
    let Some((storage, p)) = enforced().await else {
        return false;
    };
    let days = p.config.password_rotation_days;
    if days == 0 {
        return false;
    }
    let now = voltrus::timeutil::now_secs();
    match last_changed(storage, &user.username, now).await {
        Ok(t) => is_expired(t, now, days),
        Err(e) => {
            tracing::error!("password age of {}: {e:#}", user.username);
            false
        }
    }
}

// ── sessions ──

pub fn session_policy_of(p: &Policy) -> SessionPolicy {
    let minutes = u64::from(p.config.auto_logout_minutes);
    let hours = u64::from(p.extras.session_max_hours);
    SessionPolicy {
        idle_timeout_secs: (minutes > 0).then_some(minutes * 60),
        absolute_timeout_secs: (hours > 0).then_some(hours * 3600),
    }
}

pub async fn session_policy() -> SessionPolicy {
    match enforced().await {
        Some((_, p)) => session_policy_of(&p),
        None => SessionPolicy::default(),
    }
}

// ── electronic signatures ──

async fn requires_reason(storage: &Storage, tag_id: i64) -> bool {
    storage
        .with_conn(move |c| {
            let flag: Option<bool> = c
                .query_row(
                    "SELECT write_requires_reason FROM data_sources WHERE id = ?1",
                    [tag_id],
                    |r| r.get(0),
                )
                .ok();
            Ok(flag.unwrap_or(false))
        })
        .await
        // A tag whose policy cannot be read is treated as in scope: asking for
        // a signature too often is the safe mistake.
        .unwrap_or(true)
}

pub async fn signature_required(ctx: &ActionContext) -> Option<SignatureRequirement> {
    let (storage, p) = enforced().await?;
    let sig = &p.extras.signatures;
    let tag = policy::tag_id_of(&ctx.resource);
    let in_scope = match (sig.tag_scope, tag) {
        (TagScope::All, _) => true,
        (TagScope::Listed, Some(id)) => sig.tag_ids.contains(&id),
        (TagScope::RequiresReason, Some(id))
            if ctx.action == voltrus::edition::SignedAction::TagWrite
                && sig.actions.tag_write.enabled =>
        {
            requires_reason(storage, id).await
        }
        (TagScope::RequiresReason, Some(_)) => false,
        // A tag write always names its tag; anything else is in scope.
        (_, None) => true,
    };
    policy::requirement_for(&p, ctx, |_| in_scope).map(|m| SignatureRequirement {
        meaning: m.to_string(),
    })
}

/// `alarm:5` -> (`alarm`, `5`); `device:3:go_live` -> (`device`, `3:go_live`).
pub fn split_resource(resource: &str) -> (String, String) {
    match resource.split_once(':') {
        Some((t, id)) => (t.to_string(), id.to_string()),
        None => (resource.to_string(), String::new()),
    }
}

fn origin_str(o: voltrus::edition::ActionOrigin) -> String {
    serde_json::to_value(o)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// The checksum of a signature's context row (who, how, and what was signed for).
pub fn context_checksum(signature_hash: &str, origin: &str, role: &str, details: &str) -> String {
    super::compute_record_checksum(&format!("{signature_hash}|{origin}|{role}|{details}"))
}

/// Store a signature the core verified, in the same `electronic_signatures`
/// records as signatures made on the E-Signatures page. Returns its id.
pub async fn record_signature(rec: &SignatureRecord) -> anyhow::Result<Option<String>> {
    let rt = crate::try_rt().ok_or_else(|| anyhow::anyhow!("the add-on is not started"))?;
    let (rtype, rid) = split_resource(&rec.resource);
    let action = rec.action.as_str().to_string();
    let hash = super::generate_signature_hash(
        &rec.username,
        &action,
        &rtype,
        &rid,
        &rec.reason,
        &rec.meaning,
        rec.signed_at,
    );
    let origin = origin_str(rec.origin);
    let details = serde_json::to_string(&rec.details)?;
    let checksum = context_checksum(&hash, &origin, &rec.role, &details);
    let (user, role, reason, meaning, at) = (
        rec.username.clone(),
        rec.role.clone(),
        rec.reason.clone(),
        rec.meaning.clone(),
        rec.signed_at,
    );
    let id = rt
        .storage
        .with_conn(move |c| {
            policy::ensure_schema(c)?;
            let full_name = super::export::full_name_of(c, &user)?;
            let tx = c.transaction()?;
            tx.execute(
                "INSERT INTO electronic_signatures (user_id, action, resource_type, resource_id, reason, meaning, signature_hash, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![user, action, rtype, rid, reason, meaning, hash, at],
            )?;
            let id = tx.last_insert_rowid();
            if let Some(name) = &full_name {
                tx.execute(
                    "INSERT OR REPLACE INTO compliance_signature_names (signature_id, full_name) VALUES (?1, ?2)",
                    rusqlite::params![id, name],
                )?;
            }
            tx.execute(
                "INSERT INTO compliance_signature_context (signature_id, origin, role, details, checksum) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![id, origin, role, details, checksum],
            )?;
            tx.commit()?;
            Ok(id)
        })
        .await?;
    Ok(Some(id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::ComplianceConfig;
    use crate::compliance::policy::PolicyExtras;

    fn check(user: &str, pw: &str) -> PasswordCheck {
        PasswordCheck {
            username: user.into(),
            role: "operator".into(),
            new_password: pw.into(),
            kind: PasswordChangeKind::SelfService,
        }
    }

    fn saved() -> Policy {
        Policy {
            saved: true,
            config: ComplianceConfig::default(),
            extras: PolicyExtras::default(),
        }
    }

    #[test]
    fn password_rules_refuse_weak_passwords_with_a_message() {
        let p = saved();
        assert_eq!(
            check_password_rules(&check("op", "Short1!"), &p).unwrap_err(),
            "Password must be at least 8 characters"
        );
        assert!(check_password_rules(&check("op", "alllower1!"), &p).is_err());
        assert!(check_password_rules(&check("op", "NoDigits!!"), &p).is_err());
        assert!(check_password_rules(&check("op", "NoSpecial11"), &p).is_err());
        assert!(check_password_rules(&check("op", "Good-Pass1"), &p).is_ok());
        // Equal to the user name, in any case.
        assert_eq!(
            check_password_rules(&check("Plant-Op1", "plant-op1"), &p).unwrap_err(),
            "Password cannot be the same as the user name"
        );
    }

    #[test]
    fn length_counts_characters_not_bytes() {
        let p = saved();
        // Seven characters, more than eight bytes.
        assert!(check_password_rules(&check("op", "Zöë-1ÄÖ"), &p).is_err());
    }

    #[test]
    fn expiry_counts_whole_rotation_periods() {
        let day = 86_400;
        assert!(!is_expired(0, 89 * day, 90));
        assert!(is_expired(0, 90 * day, 90));
        // 0 days: never.
        assert!(!is_expired(0, 10_000 * day, 0));
    }

    #[test]
    fn session_limits_come_from_the_policy() {
        let mut p = saved();
        p.config.auto_logout_minutes = 10;
        p.extras.session_max_hours = 8;
        assert_eq!(
            session_policy_of(&p),
            SessionPolicy {
                idle_timeout_secs: Some(600),
                absolute_timeout_secs: Some(28_800)
            }
        );
        p.extras.session_max_hours = 0;
        assert_eq!(session_policy_of(&p).absolute_timeout_secs, None);
    }

    #[test]
    fn resources_split_into_type_and_id() {
        assert_eq!(split_resource("alarm:5"), ("alarm".into(), "5".into()));
        assert_eq!(split_resource("alarm:*"), ("alarm".into(), "*".into()));
        assert_eq!(
            split_resource("device:3:go_live"),
            ("device".into(), "3:go_live".into())
        );
        assert_eq!(split_resource("x"), ("x".into(), String::new()));
    }
}
