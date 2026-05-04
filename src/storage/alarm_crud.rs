use super::Storage;
use anyhow::{Context, Result};

use crate::models::Alarm;

impl Storage {
    // ── Alarm CRUD & queries ──

    pub async fn create_alarm(
        &self,
        source_key: &str,
        severity: &str,
        value: f64,
        threshold: f64,
        message: &str,
        deadband: f64,
    ) -> Result<i64> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        let severity = severity.to_string();
        let message = message.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let now = crate::timeutil::now_secs();
            let priority = match severity.as_str() {
                "critical" => "critical",
                "info" => "info",
                _ => "warning",
            };
            conn.execute(
                "INSERT INTO alarms (source_key, severity, value, threshold, message, triggered_at, priority, state, deadband)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', ?8)",
                rusqlite::params![source_key, severity, value, threshold, message, now, priority, deadband],
            )
            .context("Insert alarm failed")?;
            let id = conn.last_insert_rowid();
            // Create audit entry
            let _ = conn.execute(
                "INSERT INTO alarm_audit_log (alarm_id, action, performed_by, details, created_at) VALUES (?1, 'created', 'system', ?2, ?3)",
                rusqlite::params![id, format!("{{\"severity\":\"{}\",\"value\":{},\"threshold\":{}}}", severity, value, threshold), now],
            );
            Ok(id)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn count_active_alarms(&self) -> Result<i64> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms
                     WHERE acknowledged_at IS NULL AND resolved_at IS NULL",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            Ok(count)
        })
        .await
        .context("Task join failed")?
    }

    /// Check if there is any unresolved alarm for this source_key+severity.
    /// Uses `resolved_at IS NULL` (regardless of acknowledgment status) so that
    /// a new alarm cannot fire until the previous one is fully resolved.
    /// This implements non-repeating alarm behavior.
    pub async fn has_unresolved_alarm(&self, source_key: &str, severity: &str) -> Result<bool> {
        let conn = self.conn.clone();
        let source_key = source_key.to_string();
        let severity = severity.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM alarms
                     WHERE source_key = ?1 AND severity = ?2
                       AND resolved_at IS NULL",
                    rusqlite::params![source_key, severity],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            Ok(count > 0)
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_alarms(&self, status: Option<&str>, limit: i64) -> Result<Vec<Alarm>> {
        let conn = self.conn.clone();
        let status = status.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &status {
                Some(s) if s == "active" => (
                    "SELECT id, source_key, COALESCE(source_name, ''), severity, COALESCE(priority, severity), value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until,
                            shelved_until, shelved_by, shelved_reason, COALESCE(deadband, 0), COALESCE(state, 'active')
                     FROM alarms
                     WHERE acknowledged_at IS NULL
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
                Some(s) if s == "acknowledged" => (
                    "SELECT id, source_key, COALESCE(source_name, ''), severity, COALESCE(priority, severity), value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until,
                            shelved_until, shelved_by, shelved_reason, COALESCE(deadband, 0), COALESCE(state, 'active')
                     FROM alarms
                     WHERE acknowledged_at IS NOT NULL
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
                _ => (
                    "SELECT id, source_key, COALESCE(source_name, ''), severity, COALESCE(priority, severity), value, threshold, message,
                            triggered_at, resolved_at, acknowledged_at, acknowledged_by, cooldown_until,
                            shelved_until, shelved_by, shelved_reason, COALESCE(deadband, 0), COALESCE(state, 'active')
                     FROM alarms
                     ORDER BY triggered_at DESC LIMIT ?1",
                    vec![Box::new(limit)],
                ),
            };
            let mut stmt = conn.prepare(sql).context("Prepare failed")?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    Ok(Alarm {
                        id: row.get(0)?,
                        source_key: row.get(1)?,
                        source_name: row.get(2)?,
                        severity: row.get(3)?,
                        priority: row.get(4)?,
                        value: row.get(5)?,
                        threshold: row.get(6)?,
                        message: row.get(7)?,
                        triggered_at: row.get(8)?,
                        resolved_at: row.get(9)?,
                        acknowledged_at: row.get(10)?,
                        acknowledged_by: row.get(11)?,
                        cooldown_until: row.get(12)?,
                        shelved_until: row.get(13)?,
                        shelved_by: row.get(14)?,
                        shelved_reason: row.get(15)?,
                        deadband: row.get(16)?,
                        state: row.get(17)?,
                    })
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }
}
