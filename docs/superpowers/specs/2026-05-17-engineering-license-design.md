# Engineering-Only License System Design

*Date: 2026-05-17*

## Problem

The current license system uses a 30-day trial that creates artificial urgency. Engineers need weeks/months to design SCADA systems. The roadmap specifies an Engineering mode: unlimited time, 2-hour data polling per restart, full config access. This replaces the trial entirely.

Additionally, engineers need to export their project from their laptop (Engineering mode) and import it into a production server (licensed).

## Goals

1. Replace 30-day trial with unlimited Engineering mode
2. Enforce 2-hour rolling data polling window (resets on restart)
3. Add .voltrus-scada project export/import for engineering → production workflow
4. Remove all trial-related code and UI references

## Architecture

### 1. LicenseTier Enum

**File:** `src/license/tier.rs`

Replace `Trial` with `Engineering`:

```rust
pub enum LicenseTier {
    Engineering,
    Starter,
    Professional,
    Enterprise,
}
```

All feature gate methods updated:
- `from_tier_str()`: `"engineering"` → `Engineering`, fallback → `Engineering`
- `max_devices()`: Engineering → Some(5)
- `data_retention_days()`: Engineering → 7
- All `!matches!(self, LicenseTier::Trial)` → `!matches!(self, LicenseTier::Engineering)`
- `to_feature_flags()`: includes `"tier": "engineering"`

### 2. LicenseStatus Enum

**File:** `src/license/status.rs`

Replace `Trial` and `TrialExpired` variants:

```rust
pub enum LicenseStatus {
    Valid { ... },                    // unchanged
    Engineering {                     // replaces Trial
        started_at: i64,              // process start timestamp
        polling_elapsed_secs: i64,    // seconds since started_at
        polling_remaining_secs: i64, // 7200 - polling_elapsed_secs
        tier: LicenseTier,
        feature_flags: serde_json::Value,
    },
    EngineeringPollingStopped {       // replaces TrialExpired
        started_at: i64,
        message: String,
    },
    Unactivated { ... },              // unchanged
    InvalidFingerprint { ... },       // unchanged
    Expired { ... },                  // unchanged
    InvalidSignature { ... },         // unchanged
    Corrupted { ... },                // unchanged
}
```

`is_valid()`: `Valid` and `Engineering` return true, `EngineeringPollingStopped` returns true (config still works, only polling stops).

`tier()`: `Engineering` → `LicenseTier::Engineering`, `EngineeringPollingStopped` → `LicenseTier::Engineering`.

`feature_flags()`: same logic, Engineering gets Engineering tier flags.

### 3. License Verification

**File:** `src/license/verification.rs`

`resolve_license_status()`:
- Try paid license first (existing `verify_license()` flow)
- If no paid license → return `Engineering` status
- Calculate `polling_elapsed_secs` from `process_start_time` (passed from main.rs)
- If `polling_elapsed_secs >= 7200` → return `EngineeringPollingStopped`
- Remove all trial logic: no `trial_start` settings, no `TRIAL_DURATION_DAYS`

### 4. Polling Enforcement

**File:** `src/main.rs`

- Capture `process_start_time` at startup: `let process_start = std::time::SystemTime::now();`
- Pass to `resolve_license_status()` via a new parameter or AppState
- Before starting Modbus/OPC-UA/S7/etc pollers:
  - Check elapsed time < 7200 seconds
  - If yes: start pollers normally
  - If no: log warning, skip poller startup
- Add background checker task:
  - Runs every 60 seconds
  - Checks if polling time exceeded
  - If exceeded: signal pollers to stop via `Arc<AtomicBool>` shutdown flag
  - Pollers check this flag each poll cycle and exit gracefully

### 5. .voltrus-scada Export

**File:** `src/cli.rs` (new CLI subcommand)

`voltrus export --output project.voltrus-scada`:
- Creates a ZIP file containing:
  - `config.sqlite` — full SQLite database dump (devices, tags, screens, alarms, flows, users, settings, annotations, shift-handover, security events, asset inventory, templates, egress jobs, webhooks, RBAC roles/permissions)
  - `manifest.json` — metadata (export timestamp, Voltrus version, tier at export, device count, tag count)
- Excludes: password hashes, API keys, activation blobs, trial/engineering state
- CLI: `voltrus export [--output <path>]`
- Default output: `./voltrus-project.voltrus-scada`

### 6. .voltrus-scada Import

**File:** `src/cli.rs` (new CLI subcommand)

`voltrus import project.voltrus-scada`:
- Validates ZIP structure and manifest
- Checks target server's license tier against exported features:
  - If exported config uses Enterprise features but target is Starter → warn, skip those features
  - If target has no license (Engineering mode) → import everything, features gated at runtime
- Imports into existing SQLite database:
  - Devices, tags, screens, alarms, flows, settings, annotations, RBAC roles
  - Users imported but passwords reset (must_change_password = true)
- CLI: `voltrus import <path> [--force] [--dry-run]`
- `--dry-run`: show what would be imported without making changes
- `--force`: overwrite existing config with same names

### 7. Frontend Changes

**Files:** `client/src/` components

- License status banner:
  - Engineering mode: "Engineering Mode — Xh Ym polling remaining"
  - Polling stopped: "Data polling stopped (2h limit reached). Restart Voltrus to continue polling. Config remains fully editable."
  - Paid license: unchanged
- Settings page: show "Engineering Mode" badge with info tooltip explaining the mode
- Remove any trial countdown, "30 days remaining" text, trial expiration warnings
- Update any "Activate License" CTAs to be clearer about Engineering vs Production

### 8. Database Changes

**File:** `src/storage/`

- Remove `trial_start` setting key handling
- No new tables needed for export/import (uses existing schema)
- License activation table unchanged

## Data Flow

### Engineering Mode Startup
```
main.rs starts → capture process_start_time
→ resolve_license_status() → no paid license found
→ return Engineering { started_at, elapsed, remaining, tier, flags }
→ AppState { license_status, license_tier, feature_flags }
→ Check elapsed < 7200? → Yes: start pollers
→ No: skip pollers, config still accessible
→ Background checker: every 60s, signal pollers to stop if exceeded
```

### Export Flow
```
voltrus export → open SQLite connection
→ CREATE backup to temp file (VACUUM into backup)
→ Read device count, tag count, screen count
→ Write manifest.json
→ ZIP config.sqlite + manifest.json → output path
→ Print summary: "Exported X devices, Y tags, Z screens"
```

### Import Flow
```
voltrus import <path> → validate ZIP
→ Read manifest.json
→ Check target tier vs exported features
→ If dry-run: print what would be imported
→ Else: ATTACH backup DB, INSERT OR REPLACE into main DB
→ Reset user passwords
→ Print summary: "Imported X devices, Y tags, Z screens"
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Export with no config | Warning, export empty manifest |
| Import into Engineering mode | All features imported, gated at runtime |
| Import Enterprise config into Starter | Warn, skip Enterprise-only features (S7, EtherNet/IP, etc.) |
| Corrupted ZIP | Error: "Invalid project file" |
| Import with --force | Overwrite existing entities by name |
| Polling limit reached mid-session | Graceful poller shutdown, banner shown in UI |

## Testing

- Unit tests: `LicenseTier::Engineering` feature flags match old Trial flags
- Unit tests: `LicenseStatus::Engineering` is_valid() = true
- Unit tests: polling elapsed calculation
- Integration tests: export creates valid ZIP with manifest
- Integration tests: import restores config from ZIP
- Integration tests: import with tier mismatch skips gated features
- Manual test: start in Engineering mode, wait 2h, verify polling stops
- Manual test: restart after polling stops, verify polling resumes

## Migration

Existing installations with trial state:
- `trial_start` setting is ignored on startup
- `resolve_license_status()` falls through to Engineering mode
- No data loss — trial config is preserved, just the time limit changes
