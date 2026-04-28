use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::models::RetentionConfig;
use crate::storage::Storage;
use crate::timeutil;

pub async fn run_retention_task(storage: Storage, config: RetentionConfig) {
    if !config.enabled {
        tracing::info!("Data retention disabled");
        return;
    }

    tracing::info!(
        "Data retention task started (interval: {}s, archive_dir: {})",
        config.run_interval_secs,
        config.archive_dir,
    );

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(config.run_interval_secs));

    loop {
        interval.tick().await;

        if let Err(e) = run_retention_cycle(&storage, &config).await {
            tracing::error!("Retention cycle failed: {e}");
        }
    }
}

async fn run_retention_cycle(storage: &Storage, config: &RetentionConfig) -> Result<()> {
    let now = timeutil::now_secs();
    tracing::info!("Starting retention cycle");

    let raw_keep = get_tier_keep(&config.tiers, "raw");
    let hourly_keep = get_tier_keep(&config.tiers, "hourly");
    let daily_keep = get_tier_keep(&config.tiers, "daily");
    let should_archive = get_tier_archive(&config.tiers, "raw");

    // Step 1: Downsample raw -> hourly
    if let Some(raw_keep_secs) = raw_keep {
        let cutoff = now - raw_keep_secs as i64;
        tracing::info!("Downsampling raw telemetry older than {}s (cutoff={})", raw_keep_secs, cutoff);

        if should_archive {
            archive_raw(storage, cutoff, &config.archive_dir).await?;
        }

        let count = storage.downsample_raw_to_hourly(cutoff).await?;
        tracing::info!("Downsampled {count} raw rows to hourly");

        let deleted = storage.delete_raw_older_than(cutoff).await?;
        tracing::info!("Deleted {deleted} raw rows older than cutoff");
    }

    // Step 2: Downsample hourly -> daily
    if let Some(hourly_keep_secs) = hourly_keep {
        let cutoff = now - hourly_keep_secs as i64;
        tracing::info!("Downsampling hourly telemetry older than {}s (cutoff={})", hourly_keep_secs, cutoff);

        let count = storage.downsample_hourly_to_daily(cutoff).await?;
        tracing::info!("Downsampled {count} hourly rows to daily");

        let deleted = storage.delete_hourly_older_than(cutoff).await?;
        tracing::info!("Deleted {deleted} hourly rows older than cutoff");
    }

    // Step 3: Cleanup daily if configured
    if let Some(daily_keep_secs) = daily_keep {
        let cutoff = now - daily_keep_secs as i64;
        let deleted = storage.delete_daily_older_than(cutoff).await?;
        if deleted > 0 {
            tracing::info!("Deleted {deleted} daily rows older than cutoff");
        }
    }

    tracing::info!("Retention cycle complete");
    Ok(())
}

async fn archive_raw(storage: &Storage, cutoff: i64, archive_dir: &str) -> Result<()> {
    let rows = storage.query_raw_for_archive(cutoff).await?;
    if rows.is_empty() {
        return Ok(());
    }

    // Group rows by date
    let mut by_date: BTreeMap<String, Vec<(String, String, i64, f64)>> = BTreeMap::new();
    for (device_id, metric, ts, value) in &rows {
        let date = time::OffsetDateTime::from_unix_timestamp(*ts)
            .ok()
            .and_then(|dt| dt.format(&time::macros::format_description!("[year]-[month]-[day]")).ok())
            .unwrap_or_else(|| "unknown".to_string());
        by_date.entry(date).or_default().push((
            device_id.clone(),
            metric.clone(),
            *ts,
            *value,
        ));
    }

    std::fs::create_dir_all(archive_dir)
        .with_context(|| format!("Failed to create archive dir: {archive_dir}"))?;

    for (date, day_rows) in &by_date {
        let path = PathBuf::from(archive_dir).join(format!("{date}.csv.gz"));
        let file = std::fs::File::create(&path)
            .with_context(|| format!("Failed to create archive file: {}", path.display()))?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        writeln!(encoder, "device_id,metric,timestamp,value")?;
        for (device_id, metric, ts, value) in day_rows {
            writeln!(encoder, "{device_id},{metric},{ts},{value}")?;
        }
        encoder.finish().context("Failed to finish gzip archive")?;
        tracing::info!("Archived {} rows to {}", day_rows.len(), path.display());
    }

    Ok(())
}

fn get_tier_keep(tiers: &[crate::models::RetentionTier], name: &str) -> Option<u64> {
    tiers.iter().find(|t| t.name == name).and_then(|t| t.keep)
}

fn get_tier_archive(tiers: &[crate::models::RetentionTier], name: &str) -> bool {
    tiers.iter().find(|t| t.name == name).map(|t| t.archive).unwrap_or(false)
}
