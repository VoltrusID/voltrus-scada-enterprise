use super::Storage;
use anyhow::Context;

impl Storage {
    pub async fn compress_hot_to_warm(&self, cutoff_ms: i64) -> anyhow::Result<usize> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let rows: Vec<(i64, String, String)> = conn
                .prepare(
                    "SELECT DISTINCT device_id, metric, source_type FROM ts_data WHERE timestamp < ?1 AND tier = 'hot'",
                )?
                .query_map(rusqlite::params![cutoff_ms], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            let mut compressed = 0usize;
            for (device_id, metric, source_type) in &rows {
                let avg: f64 = conn
                    .query_row(
                        "SELECT COALESCE(AVG(value), 0) FROM ts_data WHERE device_id = ?1 AND metric = ?2 AND source_type = ?3 AND timestamp < ?4 AND tier = 'hot'",
                        rusqlite::params![device_id, metric, source_type, cutoff_ms],
                        |row| row.get(0),
                    )
                    .unwrap_or(0.0);

                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM ts_data WHERE device_id = ?1 AND metric = ?2 AND source_type = ?3 AND timestamp < ?4 AND tier = 'hot'",
                        rusqlite::params![device_id, metric, source_type, cutoff_ms],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                if count > 0 {
                    conn.execute(
                        "INSERT OR REPLACE INTO ts_data_agg (device_id, metric, source_type, timestamp, avg, min_val, max_val, count, tier) VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, ?6, 'warm')",
                        rusqlite::params![device_id, metric, source_type, cutoff_ms, avg, count],
                    )?;

                    let deleted = conn.execute(
                        "DELETE FROM ts_data WHERE device_id = ?1 AND metric = ?2 AND source_type = ?3 AND timestamp < ?4 AND tier = 'hot'",
                        rusqlite::params![device_id, metric, source_type, cutoff_ms],
                    )?;
                    compressed += deleted as usize;
                }
            }

            Ok(compressed)
        }).await.context("Task join failed")?
    }

    pub async fn compress_warm_to_cold(&self, cutoff_ms: i64) -> anyhow::Result<usize> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let rows: Vec<(i64, String, String)> = conn
                .prepare(
                    "SELECT DISTINCT device_id, metric, source_type FROM ts_data_agg WHERE timestamp < ?1 AND tier = 'warm'",
                )?
                .query_map(rusqlite::params![cutoff_ms], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            let mut compressed = 0usize;
            for (device_id, metric, source_type) in &rows {
                let avg: f64 = conn
                    .query_row(
                        "SELECT COALESCE(AVG(avg), 0) FROM ts_data_agg WHERE device_id = ?1 AND metric = ?2 AND source_type = ?3 AND timestamp < ?4 AND tier = 'warm'",
                        rusqlite::params![device_id, metric, source_type, cutoff_ms],
                        |row| row.get(0),
                    )
                    .unwrap_or(0.0);

                let count: i64 = conn
                    .query_row(
                        "SELECT SUM(count) FROM ts_data_agg WHERE device_id = ?1 AND metric = ?2 AND source_type = ?3 AND timestamp < ?4 AND tier = 'warm'",
                        rusqlite::params![device_id, metric, source_type, cutoff_ms],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                if count > 0 {
                    conn.execute(
                        "INSERT OR REPLACE INTO ts_data_agg (device_id, metric, source_type, timestamp, avg, min_val, max_val, count, tier) VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, ?6, 'cold')",
                        rusqlite::params![device_id, metric, source_type, cutoff_ms, avg, count],
                    )?;

                    let deleted = conn.execute(
                        "DELETE FROM ts_data_agg WHERE device_id = ?1 AND metric = ?2 AND source_type = ?3 AND timestamp < ?4 AND tier = 'warm'",
                        rusqlite::params![device_id, metric, source_type, cutoff_ms],
                    )?;
                    compressed += deleted as usize;
                }
            }

            Ok(compressed)
        }).await.context("Task join failed")?
    }

    pub async fn list_custom_metrics(&self) -> anyhow::Result<Vec<crate::models::CustomMetric>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, name, description, formula, unit, tag_refs, validation_error, created_at, updated_at FROM custom_metrics ORDER BY name",
            )?;
            let metrics = stmt
                .query_map([], |row| {
                    Ok(crate::models::CustomMetric {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        formula: row.get(3)?,
                        unit: row.get(4)?,
                        tag_refs: row.get(5)?,
                        validation_error: row.get(6)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();
            Ok(metrics)
        }).await.context("Task join failed")?
    }
}
