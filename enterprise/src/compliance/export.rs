//! Part 11 PDF exports (signature records, audit trail) and the signers' full names
//! they print.

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use voltrus::api::{AppError, AppState, AuthUser};
use voltrus::storage::Storage;

use super::pdf::{self, AuditEntry, ChainSummary, ExportMeta, SignatureEntry};
use super::{ElectronicSignature, signature_matches};

/// Most rows one PDF export prints; a wider range is refused with a hint to narrow it.
pub const MAX_PDF_ROWS: usize = 10_000;

/// Longest full name stored.
const MAX_NAME_CHARS: usize = 120;

const NAME_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS compliance_signer_names (
        username TEXT PRIMARY KEY,
        full_name TEXT NOT NULL,
        updated_at INTEGER NOT NULL,
        updated_by TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS compliance_signature_names (
        signature_id INTEGER PRIMARY KEY,
        full_name TEXT NOT NULL
    );";

pub(crate) fn ensure_schema(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(NAME_SCHEMA)
}

/// The full name on file for `username`, read inside an open connection.
pub(crate) fn full_name_of(
    conn: &rusqlite::Connection,
    username: &str,
) -> rusqlite::Result<Option<String>> {
    ensure_schema(conn)?;
    let mut stmt =
        conn.prepare("SELECT full_name FROM compliance_signer_names WHERE username = ?1")?;
    let mut rows = stmt.query([username])?;
    Ok(match rows.next()? {
        Some(r) => Some(r.get(0)?),
        None => None,
    })
}

fn license_forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"error": "CFR Part 11 requires an Enterprise licence"})),
    )
        .into_response()
}

fn bad_request(msg: impl Into<String>) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({"error": msg.into()}))).into_response()
}

// ── filters ──

#[derive(Debug, Deserialize, Default)]
pub struct SignatureFilter {
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    /// Unix seconds, inclusive.
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub username: Option<String>,
    /// Minutes east of UTC to print times in (the browser's offset). Default UTC.
    pub utc_offset_minutes: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AuditFilter {
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub search: Option<String>,
    pub utc_offset_minutes: Option<i32>,
}

fn non_empty(s: &Option<String>) -> Option<&str> {
    s.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Offsets run from UTC-12:00 to UTC+14:00.
fn checked_offset(minutes: Option<i32>) -> Result<i32, Response> {
    match minutes.unwrap_or(0) {
        m if (-720..=840).contains(&m) => Ok(m),
        _ => Err(bad_request(
            "utc_offset_minutes must be between -720 and 840",
        )),
    }
}

fn range_text(from: Option<i64>, to: Option<i64>, minutes: i32) -> String {
    let at = |t| pdf::format_time(t, minutes);
    match (from, to) {
        (Some(f), Some(t)) => format!("{} to {}", at(f), at(t)),
        (Some(f), None) => format!("from {}", at(f)),
        (None, Some(t)) => format!("until {}", at(t)),
        (None, None) => format!("all dates (times in {})", pdf::zone_label(minutes)),
    }
}

/// Signatures matching every given filter, oldest first.
pub fn filter_signatures(
    mut signatures: Vec<ElectronicSignature>,
    q: &SignatureFilter,
) -> Vec<ElectronicSignature> {
    let user = non_empty(&q.username);
    signatures.retain(|s| {
        q.from.is_none_or(|f| s.created_at >= f)
            && q.to.is_none_or(|t| s.created_at <= t)
            && user.is_none_or(|u| s.user_id == u)
    });
    signatures.sort_by_key(|s| (s.created_at, s.id));
    signatures
}

pub fn signature_filter_text(q: &SignatureFilter, minutes: i32) -> String {
    let mut parts = vec![range_text(q.from, q.to, minutes)];
    if let Some(u) = non_empty(&q.username) {
        parts.push(format!("signer {u}"));
    }
    if let Some(t) = non_empty(&q.resource_type) {
        parts.push(format!("record type {t}"));
    }
    if let Some(i) = non_empty(&q.resource_id) {
        parts.push(format!("record {i}"));
    }
    parts.join(", ")
}

fn audit_filter_text(q: &AuditFilter, minutes: i32) -> String {
    let mut parts = vec![range_text(q.from, q.to, minutes)];
    if let Some(u) = non_empty(&q.username) {
        parts.push(format!("user {u}"));
    }
    if let Some(a) = non_empty(&q.action) {
        parts.push(format!("action {a}"));
    }
    if let Some(s) = non_empty(&q.search) {
        parts.push(format!("containing \"{s}\""));
    }
    parts.join(", ")
}

async fn site_title(state: &AppState) -> String {
    if state.white_label() {
        let title = state
            .storage
            .get_settings(&["brand_title"])
            .await
            .ok()
            .and_then(|s| s.into_iter().next())
            .map(|s| s.value)
            .unwrap_or_default();
        if !title.trim().is_empty() {
            return title;
        }
    }
    "Voltrus SCADA".to_string()
}

fn pdf_response(filename: &str, body: Vec<u8>) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        body,
    )
        .into_response()
}

fn today() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Iso8601::DATE)
        .unwrap_or_else(|_| "export".to_string())
}

/// Full names: recorded at signing for each signature id, and on file per user now.
async fn names(
    storage: &Storage,
) -> anyhow::Result<(HashMap<i64, String>, HashMap<String, String>)> {
    storage
        .with_conn(|c| {
            ensure_schema(c)?;
            let mut at_signing = HashMap::new();
            let mut stmt =
                c.prepare("SELECT signature_id, full_name FROM compliance_signature_names")?;
            for row in stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
                let (id, name) = row?;
                at_signing.insert(id, name);
            }
            let mut on_file = HashMap::new();
            let mut stmt = c.prepare("SELECT username, full_name FROM compliance_signer_names")?;
            for row in
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            {
                let (user, name) = row?;
                on_file.insert(user, name);
            }
            Ok((at_signing, on_file))
        })
        .await
}

/// GET /api/v1/compliance/signatures/export.pdf
pub async fn export_signatures_pdf(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<SignatureFilter>,
) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
        return Ok(license_forbidden());
    }
    let minutes = match checked_offset(q.utc_offset_minutes) {
        Ok(m) => m,
        Err(r) => return Ok(r),
    };
    let all = state
        .storage
        .list_electronic_signatures(non_empty(&q.resource_type), non_empty(&q.resource_id))
        .await?;
    let signatures = filter_signatures(all, &q);
    if signatures.len() > MAX_PDF_ROWS {
        return Ok(bad_request(format!(
            "{} signatures match; a PDF holds at most {MAX_PDF_ROWS}. Narrow the date range.",
            signatures.len()
        )));
    }
    let (at_signing, on_file) = names(&state.storage).await?;
    let entries: Vec<SignatureEntry> = signatures
        .into_iter()
        .map(|sig| {
            let (full_name, name_is_current) = match at_signing.get(&sig.id) {
                Some(n) => (Some(n.clone()), false),
                None => (on_file.get(&sig.user_id).cloned(), true),
            };
            SignatureEntry {
                hash_valid: signature_matches(&sig),
                full_name,
                name_is_current,
                sig,
            }
        })
        .collect();
    let meta = ExportMeta {
        title: "Electronic signature records".into(),
        site: site_title(&state).await,
        filter: signature_filter_text(&q, minutes),
        generated_at: voltrus::timeutil::now_secs(),
        generated_by: auth.username.clone(),
        offset_minutes: minutes,
    };
    let body = pdf::signatures_pdf(&meta, &entries);
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "export_signatures_pdf".into(),
        username: auth.username.clone(),
        resource: "electronic_signature".into(),
        details: Some(format!("{} records; {}", entries.len(), meta.filter)),
    });
    Ok(pdf_response(
        &format!("electronic-signatures-{}.pdf", today()),
        body,
    ))
}

/// Audit entries matching the filter, oldest first, at most `limit + 1`.
async fn audit_rows(
    storage: &Storage,
    q: &AuditFilter,
    limit: usize,
) -> anyhow::Result<Vec<AuditEntry>> {
    let (from, to) = (q.from, q.to);
    let user = non_empty(&q.username).map(str::to_string);
    let action = non_empty(&q.action).map(str::to_string);
    let search = non_empty(&q.search).map(|s| format!("%{s}%"));
    storage
        .with_conn(move |c| {
            let mut sql = String::from(
                "SELECT id, created_at, username, action, resource, details, hash FROM audit_log WHERE 1=1",
            );
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if let Some(f) = from {
                params.push(Box::new(f));
                sql.push_str(&format!(" AND created_at >= ?{}", params.len()));
            }
            if let Some(t) = to {
                params.push(Box::new(t));
                sql.push_str(&format!(" AND created_at <= ?{}", params.len()));
            }
            if let Some(u) = user {
                params.push(Box::new(u));
                sql.push_str(&format!(" AND username = ?{}", params.len()));
            }
            if let Some(a) = action {
                params.push(Box::new(a));
                sql.push_str(&format!(" AND action = ?{}", params.len()));
            }
            if let Some(s) = search {
                params.push(Box::new(s));
                let i = params.len();
                sql.push_str(&format!(
                    " AND (details LIKE ?{i} OR resource LIKE ?{i} OR username LIKE ?{i} OR action LIKE ?{i})"
                ));
            }
            sql.push_str(&format!(" ORDER BY id LIMIT {}", limit + 1));
            let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let mut stmt = c.prepare(&sql)?;
            let rows = stmt
                .query_map(refs.as_slice(), |r| {
                    Ok(AuditEntry {
                        id: r.get(0)?,
                        created_at: r.get(1)?,
                        username: r.get(2)?,
                        action: r.get(3)?,
                        resource: r.get(4)?,
                        details: r.get(5)?,
                        hash: r.get(6)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
        .await
}

/// GET /api/v1/compliance/audit/export.pdf
pub async fn export_audit_pdf(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(q): Query<AuditFilter>,
) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
        return Ok(license_forbidden());
    }
    let minutes = match checked_offset(q.utc_offset_minutes) {
        Ok(m) => m,
        Err(r) => return Ok(r),
    };
    let entries = audit_rows(&state.storage, &q, MAX_PDF_ROWS).await?;
    if entries.len() > MAX_PDF_ROWS {
        return Ok(bad_request(format!(
            "More than {MAX_PDF_ROWS} audit entries match; a PDF holds at most {MAX_PDF_ROWS}. Narrow the date range or filter by user or action."
        )));
    }
    let report = state.storage.verify_audit_chain().await?;
    let chain = ChainSummary {
        ok: report.ok,
        checked: report.checked as u64,
        unhashed_legacy: report.unhashed_legacy as u64,
        first_bad_id: report.first_bad_id,
    };
    let meta = ExportMeta {
        title: "Audit trail".into(),
        site: site_title(&state).await,
        filter: audit_filter_text(&q, minutes),
        generated_at: voltrus::timeutil::now_secs(),
        generated_by: auth.username.clone(),
        offset_minutes: minutes,
    };
    let body = pdf::audit_pdf(&meta, &entries, &chain);
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "export_audit_pdf".into(),
        username: auth.username.clone(),
        resource: "audit_log".into(),
        details: Some(format!("{} entries; {}", entries.len(), meta.filter)),
    });
    Ok(pdf_response(&format!("audit-trail-{}.pdf", today()), body))
}

// ── signers' full names ──

/// GET /api/v1/compliance/signer-names: every account with its full name on file.
pub async fn list_signer_names(State(state): State<AppState>) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
        return Ok(license_forbidden());
    }
    let users = state.storage.list_users().await?;
    let (_, on_file) = names(&state.storage).await?;
    let list: Vec<_> = users
        .iter()
        .map(|u| {
            json!({
                "username": u.username,
                "role": u.role,
                "full_name": on_file.get(&u.username).cloned().unwrap_or_default(),
            })
        })
        .collect();
    Ok(Json(list).into_response())
}

#[derive(Debug, Deserialize)]
pub struct SignerNameBody {
    pub full_name: String,
}

/// Trim a full name and refuse one that cannot be printed as a name.
pub fn normalize_full_name(raw: &str) -> Result<String, String> {
    let name = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(format!(
            "A full name has at most {MAX_NAME_CHARS} characters"
        ));
    }
    if name.chars().any(|c| c.is_control() || c == '<' || c == '>') {
        return Err("A full name cannot contain control characters or < >".into());
    }
    Ok(name)
}

/// PUT /api/v1/compliance/signer-names/{username}: set (or clear, with an empty
/// name) the full name printed with this person's signatures from now on.
pub async fn set_signer_name(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(username): Path<String>,
    Json(body): Json<SignerNameBody>,
) -> Result<Response, AppError> {
    if !state.tier().cfr_part_11() {
        return Ok(license_forbidden());
    }
    let name = match normalize_full_name(&body.full_name) {
        Ok(n) => n,
        Err(e) => return Ok(bad_request(e)),
    };
    if state
        .storage
        .get_user_by_username(&username)
        .await?
        .is_none()
    {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "no such user"})),
        )
            .into_response());
    }
    let (user, by, value) = (username.clone(), auth.username.clone(), name.clone());
    state
        .storage
        .with_conn(move |c| {
            ensure_schema(c)?;
            if value.is_empty() {
                c.execute(
                    "DELETE FROM compliance_signer_names WHERE username = ?1",
                    [&user],
                )?;
            } else {
                c.execute(
                    "INSERT INTO compliance_signer_names (username, full_name, updated_at, updated_by) VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(username) DO UPDATE SET full_name = excluded.full_name, updated_at = excluded.updated_at, updated_by = excluded.updated_by",
                    rusqlite::params![user, value, voltrus::timeutil::now_secs(), by],
                )?;
            }
            Ok(())
        })
        .await?;
    state.audit_logger.log(voltrus::security::AuditEvent {
        action: "set_signer_name".into(),
        username: auth.username.clone(),
        resource: format!("user:{username}"),
        details: Some(if name.is_empty() {
            "cleared".into()
        } else {
            format!("full name: {name}")
        }),
    });
    Ok(Json(json!({"ok": true, "username": username, "full_name": name})).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(id: i64, user: &str, at: i64) -> ElectronicSignature {
        ElectronicSignature {
            id,
            user_id: user.into(),
            action: "sign".into(),
            resource_type: "batch".into(),
            resource_id: "B".into(),
            reason: "r".into(),
            meaning: "m".into(),
            signature_hash: String::new(),
            created_at: at,
        }
    }

    #[test]
    fn signatures_are_filtered_by_range_and_signer_oldest_first() {
        let all = vec![
            sig(3, "bob", 300),
            sig(1, "alice", 100),
            sig(2, "alice", 200),
        ];
        let q = SignatureFilter {
            from: Some(150),
            to: Some(300),
            ..Default::default()
        };
        let ids: Vec<i64> = filter_signatures(all.clone(), &q)
            .iter()
            .map(|s| s.id)
            .collect();
        assert_eq!(ids, vec![2, 3]);
        let q = SignatureFilter {
            username: Some(" alice ".into()),
            ..Default::default()
        };
        let ids: Vec<i64> = filter_signatures(all, &q).iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn the_filter_is_described_in_the_export_zone() {
        let q = SignatureFilter {
            from: Some(0),
            to: Some(86_399),
            username: Some("alice".into()),
            ..Default::default()
        };
        assert_eq!(
            signature_filter_text(&q, 420),
            "1970-01-01 07:00:00 UTC+07:00 to 1970-01-02 06:59:59 UTC+07:00, signer alice"
        );
        assert_eq!(
            signature_filter_text(&SignatureFilter::default(), 0),
            "all dates (times in UTC)"
        );
    }

    #[test]
    fn offsets_outside_the_real_zones_are_refused() {
        assert_eq!(checked_offset(None).ok(), Some(0));
        assert_eq!(checked_offset(Some(840)).ok(), Some(840));
        assert_eq!(checked_offset(Some(-720)).ok(), Some(-720));
        assert!(checked_offset(Some(841)).is_err());
        assert!(checked_offset(Some(-721)).is_err());
    }

    #[test]
    fn full_names_are_trimmed_and_checked() {
        assert_eq!(
            normalize_full_name("  Alice   Kusuma ").unwrap(),
            "Alice Kusuma"
        );
        assert_eq!(normalize_full_name("").unwrap(), "");
        assert!(normalize_full_name(&"x".repeat(MAX_NAME_CHARS + 1)).is_err());
        assert!(normalize_full_name("<script>").is_err());
        assert_eq!(
            normalize_full_name("Zoë O'Brien-Łukasz").unwrap(),
            "Zoë O'Brien-Łukasz"
        );
    }
}
