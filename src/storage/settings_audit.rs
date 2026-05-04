use super::Storage;
use anyhow::{Context, Result};

impl Storage {
    // ── Audit Log ──

    pub async fn log_audit_event(
        &self,
        action: &str,
        username: &str,
        resource: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let action = action.to_string();
        let username = username.to_string();
        let resource = resource.to_string();
        let details = details.map(|s| s.to_string());
        let now = crate::timeutil::now_secs();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO audit_log (action, username, resource, details, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![action, username, resource, details, now],
            )
            .context("Insert audit log failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    pub async fn list_audit_logs(&self, limit: i64) -> Result<Vec<(i64, String, String, String, Option<String>, i64)>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, action, username, resource, details, created_at
                 FROM audit_log
                 ORDER BY created_at DESC
                 LIMIT ?1"
            ).context("Prepare failed")?;
            let rows = stmt.query_map(rusqlite::params![limit], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            }).context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>().context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }
}
