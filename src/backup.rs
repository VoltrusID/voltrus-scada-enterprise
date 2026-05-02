use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use rusqlite::backup::Backup;
use rusqlite::Connection;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Metadata stored inside the backup archive.
#[derive(Serialize)]
struct BackupMetadata {
    version: String,
    timestamp: String,
    db_size_bytes: u64,
    voltrus_version: String,
}

/// Options for the backup operation.
pub struct BackupOptions {
    pub config_path: String,
    pub db_path: String,
    pub output_path: Option<String>,
}

/// Run the full backup workflow: SQLite snapshot -> tar.gz archive.
pub fn run_backup(opts: BackupOptions) -> Result<String> {
    // 1. Validate that the database file exists.
    if !Path::new(&opts.db_path).exists() {
        anyhow::bail!(
            "Database file not found: {}. Start the server first to create it.",
            opts.db_path
        );
    }

    // 2. Create a consistent SQLite backup to a temporary file.
    println!("Backing up database...");
    let temp_db = create_sqlite_backup(&opts.db_path)?;
    let db_size = fs::metadata(&temp_db)
        .with_context(|| format!("Failed to stat temp db: {}", temp_db))?
        .len();

    // 3. Determine output path.
    let output_path = match opts.output_path {
        Some(ref p) => p.clone(),
        None => format!(
            "voltrus-backup-{}.tar.gz",
            chrono_now_timestamp()
        ),
    };

    // 4. Build the tar.gz archive.
    println!("Creating archive: {}", output_path);
    create_archive(
        &temp_db,
        &opts.config_path,
        db_size,
        &output_path,
    )?;

    // 5. Clean up temp file.
    let _ = fs::remove_file(&temp_db);

    // 6. Report result.
    let archive_size = fs::metadata(&output_path)
        .with_context(|| format!("Failed to stat archive: {}", output_path))?
        .len();
    let size_human = format_size(archive_size);
    println!("Backup complete: {} ({})", output_path, size_human);

    Ok(output_path)
}

/// Use rusqlite's online backup API to create a consistent snapshot.
fn create_sqlite_backup(db_path: &str) -> Result<String> {
    let src = Connection::open(db_path)
        .with_context(|| format!("Failed to open database: {}", db_path))?;
    let temp_path = format!("{}.backup.tmp", db_path);
    let mut dst = Connection::open(&temp_path)
        .with_context(|| format!("Failed to create temp database: {}", temp_path))?;

    let backup = Backup::new(&src, &mut dst)
        .context("Failed to initialize SQLite backup")?;
    backup
        .run_to_completion(5, std::time::Duration::from_millis(250), None)
        .context("SQLite backup failed")?;

    Ok(temp_path)
}

/// Build a tar.gz archive containing the database backup, config, and metadata.
fn create_archive(
    temp_db_path: &str,
    config_path: &str,
    db_size_bytes: u64,
    output_path: &str,
) -> Result<()> {
    let archive_file =
        File::create(output_path).context("Failed to create archive file")?;
    let encoder = GzEncoder::new(archive_file, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);

    // Add database backup.
    builder
        .append_path_with_name(temp_db_path, "data.db")
        .context("Failed to add database to archive")?;
    println!("Archiving database...");

    // Add config file (if it exists).
    println!("Archiving config...");
    if Path::new(config_path).exists() {
        builder
            .append_path_with_name(config_path, "config.yaml")
            .context("Failed to add config to archive")?;
    } else {
        // Write an empty placeholder so the restore side can find the key.
        let mut empty = File::create("/dev/null").context("Failed to open /dev/null")?;
        writeln!(empty, "").ok();
        // Use append_data instead so we don't need a real file on disk.
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_cksum();
        builder
            .append_data(&mut header, "config.yaml", std::io::empty())
            .context("Failed to add empty config placeholder")?;
    }

    // Add metadata JSON.
    println!("Archiving metadata...");
    let metadata = BackupMetadata {
        version: "1.0".to_string(),
        timestamp: chrono_timestamp(),
        db_size_bytes,
        voltrus_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .context("Failed to serialize backup metadata")?;
    let mut meta_header = tar::Header::new_gnu();
    let meta_bytes = metadata_json.as_bytes();
    meta_header.set_size(meta_bytes.len() as u64);
    meta_header.set_cksum();
    builder
        .append_data(&mut meta_header, "backup.json", meta_bytes)
        .context("Failed to add metadata to archive")?;

    // Finish the archive (this also finishes the GzEncoder).
    builder
        .into_inner()
        .context("Failed to finish tar builder")?
        .finish()
        .context("Failed to finish gzip compression")?;

    Ok(())
}

/// Format a timestamp as YYYYMMDD-HHMMSS (local time).
fn chrono_now_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // Use time crate (already a dependency) for formatting.
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let moment = time::OffsetDateTime::from_unix_timestamp(duration.as_secs() as i64)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
        .to_offset(offset);
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        moment.year(),
        moment.month() as u8,
        moment.day(),
        moment.hour(),
        moment.minute(),
        moment.second(),
    )
}

/// ISO 8601 timestamp string.
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let moment = time::OffsetDateTime::from_unix_timestamp(duration.as_secs() as i64)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
        .to_offset(offset);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        moment.year(),
        moment.month() as u8,
        moment.day(),
        moment.hour(),
        moment.minute(),
        moment.second(),
    )
}

/// Human-readable file size.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn test_backup_creates_archive() {
        // Create a temporary directory with a SQLite database.
        let tmp_dir = tempfile::tempdir().unwrap();
        let db_path = tmp_dir.path().join("data.db");
        let config_path = tmp_dir.path().join("config.yaml");

        // Create a simple database with a table.
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT)", [])
            .unwrap();
        conn.execute("INSERT INTO test (val) VALUES ('hello')", [])
            .unwrap();
        drop(conn);

        // Write a config file.
        let mut f = File::create(&config_path).unwrap();
        writeln!(f, "server_port: 8080").unwrap();
        drop(f);

        // Run backup.
        let output_path = tmp_dir.path().join("backup.tar.gz");
        let opts = BackupOptions {
            config_path: config_path.to_str().unwrap().to_string(),
            db_path: db_path.to_str().unwrap().to_string(),
            output_path: Some(output_path.to_str().unwrap().to_string()),
        };
        let result = run_backup(opts);
        assert!(result.is_ok());

        // Verify archive exists and is non-empty.
        assert!(output_path.exists());
        let meta = fs::metadata(&output_path).unwrap();
        assert!(meta.len() > 0);

        // Extract the archive and verify contents.
        let archive_file = File::open(&output_path).unwrap();
        let decoder = flate2::read::GzDecoder::new(archive_file);
        let mut archive = tar::Archive::new(decoder);
        let restore_dir = tempfile::tempdir().unwrap();
        archive.unpack(restore_dir.path()).unwrap();

        // Verify expected files exist in the extracted archive.
        assert!(restore_dir.path().join("data.db").exists());
        assert!(restore_dir.path().join("config.yaml").exists());
        assert!(restore_dir.path().join("backup.json").exists());

        // Verify backup.json metadata.
        let content = fs::read_to_string(restore_dir.path().join("backup.json")).unwrap();
        let backup_meta: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(backup_meta["version"], "1.0");
        assert_eq!(backup_meta["voltrus_version"], env!("CARGO_PKG_VERSION"));
        assert!(backup_meta["timestamp"].as_str().unwrap().len() > 0);
        assert!(backup_meta["db_size_bytes"].as_u64().unwrap() > 0);

        // Verify config.yaml content.
        let cfg_content = fs::read_to_string(restore_dir.path().join("config.yaml")).unwrap();
        assert!(cfg_content.contains("server_port: 8080"));

        // Verify the backed-up database is readable and has our data.
        let conn2 = Connection::open(restore_dir.path().join("data.db")).unwrap();
        let val: String = conn2
            .query_row("SELECT val FROM test WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(val, "hello");
    }

    #[test]
    fn test_backup_fails_when_db_missing() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let opts = BackupOptions {
            config_path: tmp_dir.path().join("config.yaml").to_str().unwrap().to_string(),
            db_path: tmp_dir.path().join("nonexistent.db").to_str().unwrap().to_string(),
            output_path: None,
        };
        let result = run_backup(opts);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Database file not found"));
    }

    #[test]
    fn test_backup_without_config_file() {
        // Backup should work even if config file doesn't exist.
        let tmp_dir = tempfile::tempdir().unwrap();
        let db_path = tmp_dir.path().join("data.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE t (x)", []).unwrap();
        drop(conn);

        let output_path = tmp_dir.path().join("backup.tar.gz");
        let opts = BackupOptions {
            config_path: tmp_dir.path().join("nope.yaml").to_str().unwrap().to_string(),
            db_path: db_path.to_str().unwrap().to_string(),
            output_path: Some(output_path.to_str().unwrap().to_string()),
        };
        let result = run_backup(opts);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(3 * 1024 * 1024), "3.00 MB");
        assert_eq!(format_size(2 * 1024 * 1024 * 1024), "2.00 GB");
    }
}
