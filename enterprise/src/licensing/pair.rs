//! One Enterprise licence for a hot standby pair.
//!
//! The licence stays bound to one machine (the *holder*). Its partner in the pair
//! (the *borrower*) needs no activation of its own: over the authenticated node link
//! (TLS 1.3 plus the shared-secret proof, see `ha::link`) the holder sends its signed
//! activation file with every heartbeat of a replication session, in either
//! direction. The borrower checks the Ed25519 signature, the expiry and that the tier
//! includes hot standby, then runs with that licence.
//!
//! * The borrower keeps the licence for [`PAIR_GRACE_DAYS`] after the holder was last
//!   heard in a session, persisted in `ha_license_share` (an `ha_` table, so it never
//!   replicates). A failover therefore keeps every Enterprise feature on the promoted
//!   standby, across restarts, and the grace clock starts again as soon as the old
//!   primary is back as its standby.
//! * Only a node's own, fingerprint-bound activation is ever sent: a borrowed
//!   licence is never passed on, so a licence reaches one partner at a time.
//! * Only the peer of a replication session confirms it. That peer shares the
//!   borrower's database: two independent sites cannot both keep running with one
//!   licence, because reconnecting them makes one replace the other's data, and
//!   apart they run for the grace period at most.
//! * A node without a licence of its own may join a pair only as the standby and
//!   never promotes until it holds the shared licence (`promotion_blocked` says so).
//!   Which node is active is decided by the pair (epochs, witness) as before.

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::Serialize;
use time::OffsetDateTime;
use voltrus::license::LicenseStatus;
use voltrus::storage::Storage;

use super::activation::{self, LicenseActivation};
use super::verification::{evaluate_activation, evaluate_row};

/// How long a borrowed licence keeps working after the holder was last heard.
pub const PAIR_GRACE_DAYS: i64 = 30;

/// A received licence is written to the database at most this often while it does
/// not change (the confirmation time is kept in memory in between).
const PERSIST_EVERY_SECS: i64 = 60;

/// How long the check of this node's own activation is reused. Collecting the
/// machine fingerprint lists the whole system, too slow for every heartbeat.
const OWN_CACHE: Duration = Duration::from_secs(300);

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS ha_license_share (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    holder_node_id TEXT NOT NULL,
    activation_blob BLOB NOT NULL,
    confirmed_at INTEGER NOT NULL
)";

fn grace_secs() -> i64 {
    #[cfg(feature = "ha-test-node")]
    if let Some(secs) = std::env::var("VOLTRUS_TEST_PAIR_GRACE_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        return secs;
    }
    PAIR_GRACE_DAYS * 86_400
}

#[derive(Clone)]
struct Share {
    holder: String,
    blob: Vec<u8>,
    activation: LicenseActivation,
    /// Unix seconds the holder last confirmed it.
    confirmed_at: i64,
    persisted_at: i64,
}

struct State {
    /// This node's own licence status (the stored activation, judged on this machine).
    own: Option<LicenseStatus>,
    /// The activation this node may send to its peer, and when that was checked.
    own_blob: Option<(Instant, Option<Vec<u8>>)>,
    share: Option<Share>,
    last_refusal_log: Option<Instant>,
}

static STATE: Mutex<State> = Mutex::new(State {
    own: None,
    own_blob: None,
    share: None,
    last_refusal_log: None,
});

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

fn now_secs() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

fn at(secs: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(secs).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

fn covers_pair(status: &LicenseStatus) -> bool {
    matches!(status, LicenseStatus::Valid { .. }) && status.tier().ha()
}

/// Check an activation file received from the peer: signed by a trusted key, not
/// lapsed past grace, and a tier that includes hot standby. The fingerprint is the
/// holder's, so it is not compared with this machine.
pub fn verify_shared(blob: &[u8], now: OffsetDateTime) -> Result<LicenseActivation> {
    let activation = activation::verify_activation_signature(blob)?;
    let status = evaluate_activation(&activation, now);
    if !covers_pair(&status) {
        anyhow::bail!("the peer's licence does not include hot standby or has expired");
    }
    Ok(activation)
}

/// Record this node's own licence status (at boot and after every activation).
pub fn set_own(status: LicenseStatus) {
    let mut s = state();
    s.own = Some(status);
    s.own_blob = None;
}

fn own_covers_pair(s: &State) -> bool {
    s.own.as_ref().is_some_and(covers_pair)
}

/// The status the borrowed licence gives at `now`, if there is one in force.
fn borrowed_at(share: Option<&Share>, now: i64) -> Option<LicenseStatus> {
    let share = share?;
    if now - share.confirmed_at > grace_secs() {
        return None;
    }
    let status = evaluate_activation(&share.activation, at(now));
    covers_pair(&status).then_some(status)
}

/// True when this node received a licence from a pair partner, in force or not.
pub fn has_share() -> bool {
    state().share.is_some()
}

/// The licence this node runs with: its own when that includes hot standby,
/// otherwise the one its pair partner shared while still in force, otherwise its own.
pub fn effective(own: LicenseStatus) -> LicenseStatus {
    if covers_pair(&own) {
        return own;
    }
    borrowed_at(state().share.as_ref(), now_secs()).unwrap_or(own)
}

/// True while this node runs on its partner's licence.
pub fn borrowing() -> bool {
    let s = state();
    !own_covers_pair(&s) && borrowed_at(s.share.as_ref(), now_secs()).is_some()
}

/// What `GET /api/v1/ha/status` reports about the licence of this node.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ShareStatus {
    /// `own` (activated on this machine), `peer` (shared by the pair partner),
    /// `expired` (shared, but the partner was not heard for the grace period) or `none`.
    pub source: &'static str,
    pub holder_node_id: Option<String>,
    pub license_key: Option<String>,
    /// When the partner last confirmed the shared licence.
    pub confirmed_at: Option<String>,
    /// The shared licence stops at this time unless the partner is heard again.
    pub valid_until: Option<String>,
    pub grace_days: i64,
}

pub fn share_status() -> ShareStatus {
    let s = state();
    let rfc = |secs: i64| crate::ha::rfc3339(secs);
    let mut out = ShareStatus {
        source: "none",
        holder_node_id: None,
        license_key: None,
        confirmed_at: None,
        valid_until: None,
        grace_days: grace_secs() / 86_400,
    };
    if own_covers_pair(&s) {
        out.source = "own";
        return out;
    }
    if let Some(share) = &s.share {
        out.source = if borrowed_at(Some(share), now_secs()).is_some() {
            "peer"
        } else {
            "expired"
        };
        out.holder_node_id = Some(share.holder.clone());
        out.license_key = Some(share.activation.license_key.clone());
        out.confirmed_at = rfc(share.confirmed_at);
        out.valid_until = rfc(share.confirmed_at + grace_secs());
    }
    out
}

/// Load the shared licence a previous run received. A node restarted while its
/// partner is away keeps the borrowed licence for the rest of the grace period.
pub async fn load(storage: &Storage) -> Result<()> {
    let row = storage
        .with_conn(|c| {
            c.execute_batch(SCHEMA)?;
            let mut stmt = c.prepare(
                "SELECT holder_node_id, activation_blob, confirmed_at FROM ha_license_share WHERE id = 1",
            )?;
            let mut rows = stmt.query([])?;
            Ok(match rows.next()? {
                Some(r) => Some((
                    r.get::<_, String>(0)?,
                    r.get::<_, Vec<u8>>(1)?,
                    r.get::<_, i64>(2)?,
                )),
                None => None,
            })
        })
        .await?;
    let Some((holder, blob, confirmed_at)) = row else {
        return Ok(());
    };
    match activation::verify_activation_signature(&blob) {
        Ok(activation) => {
            state().share = Some(Share {
                holder,
                blob,
                activation,
                confirmed_at,
                persisted_at: confirmed_at,
            });
        }
        Err(e) => tracing::warn!("Ignoring the stored licence of the pair partner: {e:#}"),
    }
    Ok(())
}

/// Forget a shared licence (hot standby switched off).
pub async fn clear(storage: &Storage) -> Result<()> {
    state().share = None;
    storage
        .with_conn(|c| {
            c.execute_batch("DROP TABLE IF EXISTS ha_license_share")?;
            Ok(())
        })
        .await
}

/// The activation file this node may send to its partner: its own, verified on this
/// machine (signature, fingerprint, expiry) and including hot standby. Never a
/// borrowed one.
pub async fn own_shareable(storage: &Storage) -> Option<Vec<u8>> {
    {
        let s = state();
        if !own_covers_pair(&s) {
            return None;
        }
        if let Some((checked, blob)) = &s.own_blob
            && checked.elapsed() < OWN_CACHE
        {
            return blob.clone();
        }
    }
    let row = storage.get_license_activation().await.ok().flatten();
    let blob = match row {
        Some(row) => {
            let judged = tokio::task::spawn_blocking(move || {
                let fp = super::fingerprint::collect_fingerprint();
                let status = evaluate_row(&row, &fp, OffsetDateTime::now_utc());
                covers_pair(&status).then_some(row.activation_blob)
            })
            .await
            .ok()
            .flatten();
            judged
        }
        None => None,
    };
    state().own_blob = Some((Instant::now(), blob.clone()));
    blob
}

/// The pair partner `holder` sent its licence in a replication session.
/// Returns true when this node's effective licence changed.
pub async fn receive(storage: &Storage, holder: &str, blob: Vec<u8>) -> bool {
    let now = now_secs();
    let persist = {
        let mut s = state();
        if own_covers_pair(&s) {
            return false;
        }
        match s.share.as_mut() {
            Some(share) if share.blob == blob && share.holder == holder => {
                let was_valid = borrowed_at(Some(share), now).is_some();
                share.confirmed_at = now;
                if was_valid && now - share.persisted_at < PERSIST_EVERY_SECS.min(grace_secs() / 4)
                {
                    return false;
                }
                share.persisted_at = now;
                Some((share.clone(), !was_valid))
            }
            _ => None,
        }
    };
    let (share, changed) = match persist {
        Some(p) => p,
        None => match verify_shared(&blob, at(now)) {
            Ok(activation) => {
                let share = Share {
                    holder: holder.to_string(),
                    blob,
                    activation,
                    confirmed_at: now,
                    persisted_at: now,
                };
                state().share = Some(share.clone());
                tracing::info!(
                    "Licence {} received from the pair partner {holder}: this node runs with it (grace {} days without the partner)",
                    share.activation.license_key,
                    grace_secs() / 86_400
                );
                (share, true)
            }
            Err(e) => {
                let mut s = state();
                if s.last_refusal_log
                    .is_none_or(|t| t.elapsed() > Duration::from_secs(60))
                {
                    s.last_refusal_log = Some(Instant::now());
                    tracing::warn!("Refused the licence sent by the pair partner {holder}: {e:#}");
                }
                return false;
            }
        },
    };
    let (holder, blob, confirmed_at) = (share.holder, share.blob, share.confirmed_at);
    if let Err(e) = storage
        .with_conn(move |c| {
            c.execute_batch(SCHEMA)?;
            c.execute(
                "INSERT OR REPLACE INTO ha_license_share (id, holder_node_id, activation_blob, confirmed_at) VALUES (1, ?1, ?2, ?3)",
                rusqlite::params![holder, blob, confirmed_at],
            )?;
            Ok(())
        })
        .await
    {
        tracing::warn!("Could not store the licence of the pair partner: {e:#}");
    }
    if changed {
        publish().await;
    }
    changed
}

/// True when the live licence status differs from what [`effective`] gives now:
/// a licence just received, or a borrowed one whose grace period ended.
fn differs(current: &LicenseStatus, fresh: &LicenseStatus) -> bool {
    serde_json::to_value(current).ok() != serde_json::to_value(fresh).ok()
}

/// Bring the running server's licence status in line with [`effective`]. Called
/// at start, after a licence arrives and periodically, so a lapsed grace period
/// switches the add-on features off (the core keeps running) and a returning
/// partner switches them on again without a restart.
pub async fn publish() {
    let Some(rt) = crate::try_rt() else { return };
    let Some(own) = state().own.clone() else {
        return;
    };
    let fresh = effective(own);
    let gained = {
        let mut current = rt.license_status.write().unwrap_or_else(|e| e.into_inner());
        if !differs(&current, &fresh) {
            return;
        }
        let gained = covers_pair(&fresh) && !covers_pair(&current);
        if covers_pair(&current) && !covers_pair(&fresh) {
            tracing::error!(
                "The licence shared by the pair partner is no longer in force (partner not heard for {} days, or the licence lapsed): add-on features are off until the partner is back or a licence is activated on this node",
                grace_secs() / 86_400
            );
        }
        *current = fresh;
        gained
    };
    if gained {
        rt.apply_license_effects().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn share(confirmed_at: i64, tier: &str, expires_at: Option<&str>) -> Share {
        Share {
            holder: "node-a".into(),
            blob: vec![],
            activation: LicenseActivation {
                version: 2,
                license_key: "VLT-PAIR".into(),
                fingerprint_hash: "sha256:holder".into(),
                issued_at: "2026-01-01T00:00:00Z".into(),
                expires_at: expires_at.map(str::to_string),
                tier: Some(tier.into()),
                device_limit: None,
                site_id: None,
                site_name: None,
                support_until: None,
                signature: String::new(),
            },
            confirmed_at,
            persisted_at: confirmed_at,
        }
    }

    const T0: i64 = 1_800_000_000;
    const DAY: i64 = 86_400;

    #[test]
    fn a_borrowed_licence_holds_for_the_grace_period_and_not_longer() {
        let s = share(T0, "enterprise", None);
        assert!(borrowed_at(Some(&s), T0).is_some());
        assert!(borrowed_at(Some(&s), T0 + PAIR_GRACE_DAYS * DAY).is_some());
        assert!(borrowed_at(Some(&s), T0 + PAIR_GRACE_DAYS * DAY + 1).is_none());
        assert!(borrowed_at(None, T0).is_none());
        let status = borrowed_at(Some(&s), T0 + DAY).unwrap();
        assert!(status.tier().ha() && status.tier().cfr_part_11());
    }

    #[test]
    fn only_a_tier_with_hot_standby_is_borrowed() {
        assert!(borrowed_at(Some(&share(T0, "integrator", None)), T0).is_none());
        assert!(borrowed_at(Some(&share(T0, "starter", None)), T0).is_none());
    }

    #[test]
    fn a_lapsed_monthly_licence_is_not_borrowed_past_its_own_grace() {
        // Expired 2027-01-15; its own 7 day grace ends on the 22nd.
        let s = share(T0, "enterprise", Some("2027-01-15T00:00:00Z"));
        let jan_20 = OffsetDateTime::parse(
            "2027-01-20T00:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let jan_23 = jan_20 + time::Duration::days(3);
        let s2 = Share {
            confirmed_at: jan_20.unix_timestamp(),
            ..s.clone()
        };
        assert!(borrowed_at(Some(&s2), jan_20.unix_timestamp()).is_some());
        let s3 = Share {
            confirmed_at: jan_23.unix_timestamp(),
            ..s
        };
        assert!(borrowed_at(Some(&s3), jan_23.unix_timestamp()).is_none());
    }

    #[test]
    fn shared_files_must_carry_a_trusted_signature() {
        let now = OffsetDateTime::now_utc();
        assert!(verify_shared(b"VOLACT\x01{}", now).is_err());
        assert!(verify_shared(b"not an activation", now).is_err());
    }

    #[cfg(feature = "ha-test-node")]
    #[test]
    fn shared_files_are_judged_by_tier_and_expiry_but_not_by_this_machine() {
        let now = OffsetDateTime::now_utc();
        let fp = "sha256:another-machine";
        assert!(
            verify_shared(&activation::test_support::sign(fp, "enterprise", None), now).is_ok()
        );
        assert!(
            verify_shared(&activation::test_support::sign(fp, "integrator", None), now).is_err()
        );
        assert!(
            verify_shared(
                &activation::test_support::sign(fp, "enterprise", Some("2020-01-01T00:00:00Z")),
                now
            )
            .is_err()
        );
        // A tampered tier breaks the signature.
        let forged =
            String::from_utf8_lossy(&activation::test_support::sign(fp, "integrator", None))
                .replace("integrator", "enterprise")
                .into_bytes();
        assert!(verify_shared(&forged, now).is_err());
    }

    #[test]
    fn only_a_valid_tier_with_hot_standby_covers_the_pair() {
        let own = LicenseStatus::Valid {
            license_key: "VLT-OWN".into(),
            expires_at: None,
            days_remaining: None,
            tier: Some("enterprise".into()),
            grace: false,
            legacy_white_label: false,
            site: None,
            support_until: None,
        };
        assert!(covers_pair(&own));
        assert!(!covers_pair(&LicenseStatus::Community));
        let integrator = LicenseStatus::Valid {
            license_key: "VLT-INT".into(),
            expires_at: None,
            days_remaining: None,
            tier: Some("integrator".into()),
            grace: false,
            legacy_white_label: false,
            site: None,
            support_until: None,
        };
        assert!(!covers_pair(&integrator));
    }
}
