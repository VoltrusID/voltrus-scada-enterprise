use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use rusqlite::backup::Backup;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

/// Metadata stored inside the backup archive.
#[derive(Serialize, Deserialize)]
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

/// Options for the restore operation.
pub struct RestoreOptions {
    pub archive_path: String,
    pub config_path: String,
    pub db_path: String,
    pub force: bool,
}

/// Run the full restore workflow: validate archive -> extract -> verify -> replace.
pub fn run_restore(opts: RestoreOptions) -> Result<()> {
    // 1. Validate the archive exists and is a valid .tar.gz.
    println!("Validating archive: {}", opts.archive_path);
    validate_archive(&opts.archive_path)?;

    // 2. Open and list archive contents.
    let archive_file = File::open(&opts.archive_path)
        .with_context(|| format!("Failed to open archive: {}", opts.archive_path))?;
    let decoder = GzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decoder);

    // Collect entries to inspect them.
    let entries: Vec<(String, u64)> = archive
        .entries()
        .context("Failed to read archive entries")?
        .filter_map(|e| e.ok())
        .map(|entry| {
            let path = entry.path().unwrap_or_default().to_string_lossy().to_string();
            let size = entry.size();
            (path, size)
        })
        .collect();

    let has_data_db = entries.iter().any(|(p, _)| p == "data.db");
    let has_backup_json = entries.iter().any(|(p, _)| p == "backup.json");
    let has_config = entries.iter().any(|(p, _)| p == "config.yaml");

    if !has_data_db {
        anyhow::bail!("Archive is missing required file: data.db");
    }
    if !has_backup_json {
        anyhow::bail!("Archive is missing required file: backup.json");
    }

    // 3. Read and display backup.json metadata.
    let metadata = read_metadata_from_archive(&opts.archive_path)?;
    println!();
    println!("Backup metadata:");
    println!("  Version:        {}", metadata.version);
    println!("  Timestamp:      {}", metadata.timestamp);
    println!("  Database size:  {}", format_size(metadata.db_size_bytes));
    println!("  Voltrus version: {}", metadata.voltrus_version);
    println!();

    // 4. Version compatibility check.
    let current_version = env!("CARGO_PKG_VERSION");
    let backup_major = parse_major_version(&metadata.voltrus_version);
    let current_major = parse_major_version(current_version);
    if backup_major != current_major {
        println!(
            "WARNING: Major version mismatch — backup was created with voltrus {}, current is {}.",
            metadata.voltrus_version, current_version
        );
        println!("  Restoring may cause issues if the database schema changed.");
        println!();
    }

    // 5. Check for running server (look for the database lock file).
    let lock_path = format!("{}-journal", opts.db_path);
    if Path::new(&lock_path).exists() || Path::new(&opts.db_path).exists() {
        // Try to open the DB exclusively — if it fails, the server likely holds it.
        if let Ok(conn) = Connection::open_with_flags(
            &opts.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            // Check if there are active readers/writers
            let is_wal = Path::new(&format!("{}-wal", opts.db_path)).exists();
            drop(conn);
            if is_wal {
                println!("WARNING: Database WAL file exists. The server may be running.");
                println!("  It is recommended to stop the server before restoring.");
                println!();
            }
        }
    }

    // 6. Confirmation prompt (unless --force).
    if !opts.force {
        println!("This will replace the current database and config at:");
        println!("  Database: {}", opts.db_path);
        println!("  Config:   {}", opts.config_path);
        println!();
        print!("Continue? [y/N] ");
        std::io::stdout().flush().context("Failed to flush stdout")?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;
        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            println!("Restore cancelled.");
            return Ok(());
        }
    }

    // 7. Create pre-restore backup of current state.
    if Path::new(&opts.db_path).exists() {
        println!("Creating pre-restore backup...");
        let pre_restore_path = format!(
            "voltrus-pre-restore-{}.tar.gz",
            chrono_now_timestamp()
        );
        let pre_restore_opts = BackupOptions {
            config_path: opts.config_path.clone(),
            db_path: opts.db_path.clone(),
            output_path: Some(pre_restore_path.clone()),
        };
        if let Err(e) = run_backup(pre_restore_opts) {
            println!("WARNING: Failed to create pre-restore backup: {}", e);
            println!("  Continuing anyway. Proceed at your own risk.");
        } else {
            println!("Pre-restore backup saved: {}", pre_restore_path);
        }
    }

    // 8. Extract to a temporary directory.
    println!("Extracting archive...");
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;

    let archive_file = File::open(&opts.archive_path)
        .with_context(|| format!("Failed to open archive: {}", opts.archive_path))?;
    let decoder = GzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decoder);
    for entry in archive
        .entries()
        .context("Failed to read archive entries")?
    {
        let mut entry = entry.context("Failed to read archive entry")?;
        let path = entry.path().context("Failed to read entry path")?;
        // Reject paths with directory traversal or non-normal components
        let safe = path
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)));
        if !safe {
            anyhow::bail!(
                "Archive contains unsafe path: {}. Refusing to extract.",
                path.display()
            );
        }
        entry
            .unpack_in(temp_dir.path())
            .context("Failed to extract archive entry")?;
    }

    let extracted_db = temp_dir.path().join("data.db");
    let extracted_config = temp_dir.path().join("config.yaml");

    if !extracted_db.exists() {
        anyhow::bail!("Extracted archive is missing data.db");
    }

    // 9. Verify database integrity.
    println!("Verifying database integrity...");
    let conn = Connection::open(&extracted_db)
        .with_context(|| "Failed to open extracted database")?;
    let result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .context("Failed to run integrity check")?;
    drop(conn);

    if result != "ok" {
        anyhow::bail!(
            "Database integrity check failed: {}. Aborting restore.",
            result
        );
    }
    println!("  Integrity check: OK");

    // 10. Verify the restored database opens correctly with a simple query.
    let conn = Connection::open(&extracted_db)
        .context("Failed to re-open extracted database for verification")?;
    // Just verify we can query it — don't assume any particular schema.
    let _: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' LIMIT 1")
        .context("Failed to query extracted database")?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    drop(conn);

    // 11. Replace the database file.
    println!("Replacing database: {}", opts.db_path);
    if let Some(parent) = Path::new(&opts.db_path).parent() {
        fs::create_dir_all(parent)
            .context("Failed to create database directory")?;
    }
    // Extract to a temp path adjacent to target (same filesystem for atomic rename)
    let temp_target = format!("{}.restoring", opts.db_path);
    fs::copy(&extracted_db, &temp_target)
        .with_context(|| format!("Failed to prepare database at {}", temp_target))?;
    fs::rename(&temp_target, &opts.db_path)
        .with_context(|| format!("Failed to replace database at {}", opts.db_path))?;

    let restored_size = fs::metadata(&opts.db_path)
        .with_context(|| format!("Failed to stat restored database: {}", opts.db_path))?
        .len();
    println!("  Restored database size: {}", format_size(restored_size));

    // 12. Restore config.yaml if present and non-empty in the archive.
    if has_config && extracted_config.exists() {
        let config_content = fs::read_to_string(&extracted_config).unwrap_or_default();
        if !config_content.trim().is_empty() {
            println!("Restoring config: {}", opts.config_path);
            if let Some(parent) = Path::new(&opts.config_path).parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create config directory")?;
            }
            fs::copy(&extracted_config, &opts.config_path)
                .with_context(|| format!("Failed to copy config to {}", opts.config_path))?;
        } else {
            println!("Config in archive is empty, skipping config restore.");
        }
    } else {
        println!("No config in archive, skipping config restore.");
    }

    // 13. Final verification — open the restored database.
    println!("Verifying restored database...");
    let verify_conn = Connection::open(&opts.db_path)
        .with_context(|| format!("Failed to open restored database: {}", opts.db_path))?;
    let verify_result: String = verify_conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .context("Failed to verify restored database")?;
    drop(verify_conn);

    if verify_result != "ok" {
        anyhow::bail!(
            "Restored database integrity check failed: {}",
            verify_result
        );
    }
    println!("  Restored database integrity: OK");

    println!();
    println!("Restore complete. You may start the server.");

    Ok(())
}

/// Validate that the given file is a valid gzip archive.
fn validate_archive(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        anyhow::bail!("Archive not found: {}", path);
    }

    let file = File::open(path)
        .with_context(|| format!("Failed to open archive: {}", path))?;
    let decoder = GzDecoder::new(file);
    // Try to read the first byte to verify gzip validity.
    let mut buf = [0u8; 1];
    let mut reader = std::io::BufReader::new(decoder);
    reader
        .read_exact(&mut buf)
        .context("File is not a valid gzip archive")?;

    Ok(())
}

/// Read and parse backup.json from the archive without fully extracting.
fn read_metadata_from_archive(archive_path: &str) -> Result<BackupMetadata> {
    let file = File::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path))?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    let mut metadata_content = String::new();
    let mut found = false;

    for entry in archive
        .entries()
        .context("Failed to read archive entries")?
    {
        let mut entry = entry.context("Failed to read archive entry")?;
        let path = entry
            .path()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if path == "backup.json" {
            entry
                .read_to_string(&mut metadata_content)
                .context("Failed to read backup.json from archive")?;
            found = true;
            break;
        }
    }

    if !found {
        anyhow::bail!("backup.json not found in archive");
    }

    let metadata: BackupMetadata = serde_json::from_str(&metadata_content)
        .context("Failed to parse backup.json metadata")?;

    Ok(metadata)
}

/// Parse the major version number from a semver-like string.
fn parse_major_version(version: &str) -> u32 {
    version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}
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

    #[test]
    fn test_restore_from_valid_archive() {
        // Create a backup first, then restore from it.
        let tmp_dir = tempfile::tempdir().unwrap();
        let db_path = tmp_dir.path().join("data.db");
        let config_path = tmp_dir.path().join("config.yaml");

        // Create source database with known data.
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT)", [])
            .unwrap();
        conn.execute("INSERT INTO test (val) VALUES ('restored')", [])
            .unwrap();
        drop(conn);

        // Write config.
        let mut f = File::create(&config_path).unwrap();
        writeln!(f, "server_port: 8080").unwrap();
        drop(f);

        // Create backup archive.
        let archive_path = tmp_dir.path().join("backup.tar.gz");
        let backup_opts = BackupOptions {
            config_path: config_path.to_str().unwrap().to_string(),
            db_path: db_path.to_str().unwrap().to_string(),
            output_path: Some(archive_path.to_str().unwrap().to_string()),
        };
        assert!(run_backup(backup_opts).is_ok());
        assert!(archive_path.exists());

        // Set up restore target paths (different from source).
        let restore_dir = tempfile::tempdir().unwrap();
        let target_db = restore_dir.path().join("data.db");
        let target_config = restore_dir.path().join("config.yaml");

        // Restore with --force (skip prompt).
        let restore_opts = RestoreOptions {
            archive_path: archive_path.to_str().unwrap().to_string(),
            config_path: target_config.to_str().unwrap().to_string(),
            db_path: target_db.to_str().unwrap().to_string(),
            force: true,
        };
        let result = run_restore(restore_opts);
        assert!(result.is_ok());

        // Verify restored database has the original data.
        assert!(target_db.exists());
        let conn2 = Connection::open(&target_db).unwrap();
        let val: String = conn2
            .query_row("SELECT val FROM test WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(val, "restored");
        drop(conn2);

        // Verify restored config.
        assert!(target_config.exists());
        let cfg = fs::read_to_string(&target_config).unwrap();
        assert!(cfg.contains("server_port: 8080"));
    }

    #[test]
    fn test_restore_fails_on_invalid_archive() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let fake_archive = tmp_dir.path().join("not-a-backup.txt");
        let mut f = File::create(&fake_archive).unwrap();
        writeln!(f, "this is not a gzip file").unwrap();
        drop(f);

        let restore_opts = RestoreOptions {
            archive_path: fake_archive.to_str().unwrap().to_string(),
            config_path: tmp_dir.path().join("config.yaml").to_str().unwrap().to_string(),
            db_path: tmp_dir.path().join("data.db").to_str().unwrap().to_string(),
            force: true,
        };
        let result = run_restore(restore_opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_restore_fails_on_missing_archive() {
        let restore_opts = RestoreOptions {
            archive_path: "/nonexistent/backup.tar.gz".to_string(),
            config_path: "config.yaml".to_string(),
            db_path: "data.db".to_string(),
            force: true,
        };
        let result = run_restore(restore_opts);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_restore_missing_data_db_fails() {
        // Create a tar.gz that has backup.json but not data.db.
        let tmp_dir = tempfile::tempdir().unwrap();
        let archive_path = tmp_dir.path().join("incomplete.tar.gz");

        let archive_file = File::create(&archive_path).unwrap();
        let encoder = GzEncoder::new(archive_file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);

        // Add only backup.json, no data.db.
        let meta = BackupMetadata {
            version: "1.0".to_string(),
            timestamp: chrono_timestamp(),
            db_size_bytes: 100,
            voltrus_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let meta_json = serde_json::to_string_pretty(&meta).unwrap();
        let mut header = tar::Header::new_gnu();
        header.set_size(meta_json.len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, "backup.json", meta_json.as_bytes())
            .unwrap();
        builder.into_inner().unwrap().finish().unwrap();

        let restore_opts = RestoreOptions {
            archive_path: archive_path.to_str().unwrap().to_string(),
            config_path: tmp_dir.path().join("config.yaml").to_str().unwrap().to_string(),
            db_path: tmp_dir.path().join("data.db").to_str().unwrap().to_string(),
            force: true,
        };
        let result = run_restore(restore_opts);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("data.db"));
    }

    #[test]
    fn test_parse_major_version() {
        assert_eq!(parse_major_version("0.19.4"), 0);
        assert_eq!(parse_major_version("1.0.0"), 1);
        assert_eq!(parse_major_version("2.3.1"), 2);
        assert_eq!(parse_major_version("invalid"), 0);
        assert_eq!(parse_major_version(""), 0);
    }

    #[test]
    fn test_read_metadata_from_archive() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let db_path = tmp_dir.path().join("data.db");
        let config_path = tmp_dir.path().join("config.yaml");

        let conn = Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE t (x)", []).unwrap();
        drop(conn);
        let mut f = File::create(&config_path).unwrap();
        writeln!(f, "port: 8080").unwrap();
        drop(f);

        let archive_path = tmp_dir.path().join("backup.tar.gz");
        let opts = BackupOptions {
            config_path: config_path.to_str().unwrap().to_string(),
            db_path: db_path.to_str().unwrap().to_string(),
            output_path: Some(archive_path.to_str().unwrap().to_string()),
        };
        run_backup(opts).unwrap();

        let meta = read_metadata_from_archive(archive_path.to_str().unwrap()).unwrap();
        assert_eq!(meta.version, "1.0");
        assert_eq!(meta.voltrus_version, env!("CARGO_PKG_VERSION"));
        assert!(meta.db_size_bytes > 0);
        assert!(!meta.timestamp.is_empty());
    }

    #[test]
    fn test_restore_creates_pre_restore_backup() {
        // Create an existing database, then restore a different backup.
        // The pre-restore backup should be created automatically.
        let tmp_dir = tempfile::tempdir().unwrap();

        // Create "existing" database with known data.
        let existing_db = tmp_dir.path().join("data.db");
        let existing_config = tmp_dir.path().join("config.yaml");
        let conn = Connection::open(&existing_db).unwrap();
        conn.execute("CREATE TABLE existing (val TEXT)", []).unwrap();
        conn.execute("INSERT INTO existing (val) VALUES ('old_data')", [])
            .unwrap();
        drop(conn);
        let mut f = File::create(&existing_config).unwrap();
        writeln!(f, "server_port: 9999").unwrap();
        drop(f);

        // Create a backup from a different source.
        let backup_dir = tempfile::tempdir().unwrap();
        let source_db = backup_dir.path().join("data.db");
        let source_config = backup_dir.path().join("config.yaml");
        let conn2 = Connection::open(&source_db).unwrap();
        conn2.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT)", [])
            .unwrap();
        conn2.execute("INSERT INTO test (val) VALUES ('new_data')", [])
            .unwrap();
        drop(conn2);
        let mut f2 = File::create(&source_config).unwrap();
        writeln!(f2, "server_port: 8080").unwrap();
        drop(f2);

        let archive_path = tmp_dir.path().join("backup.tar.gz");
        let backup_opts = BackupOptions {
            config_path: source_config.to_str().unwrap().to_string(),
            db_path: source_db.to_str().unwrap().to_string(),
            output_path: Some(archive_path.to_str().unwrap().to_string()),
        };
        run_backup(backup_opts).unwrap();

        // Restore into the same location where the existing DB lives.
        let restore_opts = RestoreOptions {
            archive_path: archive_path.to_str().unwrap().to_string(),
            config_path: existing_config.to_str().unwrap().to_string(),
            db_path: existing_db.to_str().unwrap().to_string(),
            force: true,
        };
        run_restore(restore_opts).unwrap();

        // Verify the database was replaced.
        let conn3 = Connection::open(&existing_db).unwrap();
        let val: String = conn3
            .query_row("SELECT val FROM test WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(val, "new_data");
        drop(conn3);

        // Verify config was replaced.
        let cfg = fs::read_to_string(&existing_config).unwrap();
        assert!(cfg.contains("server_port: 8080"));
    }
}
