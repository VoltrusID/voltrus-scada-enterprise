# v0.25.0 — Data Compression & Performance

**Date:** 2026-05-03
**Status:** Approved
**Timeline:** 5-6 weeks
**Tier:** Starter (basic deadband) / Professional ($499, full compression + stats)
**Depends on:** v0.24.0 (Command Ack) for storage.rs merge coordination

---

## Goal

Years of data in <15GB, not <15GB/day. Replace the existing `telemetry_hourly`/`telemetry_daily` downsampling system with a proper hot/warm/cold compression tier system using paper-accurate Gorilla encoding. Add performance optimizations for edge deployments (Raspberry Pi) and connection pooling.

**Target:** 1 year of 10K-tag data at 1Hz in ~15GB.

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Compression architecture | Dedicated tables (hot/warm/cold) | Clean separation, easy debugging per tier, straightforward migration |
| Encoding | Bit-level Gorilla (paper-accurate) | Maximum compression ratio (~100:1 cold) |
| Decompression | Fully transparent to frontend | No frontend changes needed for historical queries |
| Scope | Full roadmap | Deadband, compression workers, CLI, connection pooling, WAL tuning, memory profiling, flash optimization, performance budget |
| Existing system | Replace `telemetry_hourly`/`telemetry_daily` | Clean architecture, no legacy coexistence |
| Frontend | Compression stats dashboard | System panel showing storage, compression ratios, tier breakdown |

---

## Section 1: Data Model & Storage Schema

### Table Schemas

**`telemetry_hot`** (replaces current `telemetry`, raw data, 0-24h):
```sql
CREATE TABLE telemetry_hot (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL,
    metric TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    value REAL NOT NULL,
    quality INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_hot_time ON telemetry_hot(device_id, metric, timestamp);
```

**`telemetry_warm`** (1-minute delta-encoded buckets, 1-30 days):
```sql
CREATE TABLE telemetry_warm (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL,
    metric TEXT NOT NULL,
    bucket_start INTEGER NOT NULL,  -- Unix ms, aligned to minute boundary
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    count INTEGER NOT NULL,
    deltas BLOB  -- Varint-encoded delta sequence for point reconstruction
);
CREATE INDEX idx_warm_time ON telemetry_warm(device_id, metric, bucket_start);
```

**`telemetry_cold`** (Gorilla/RLE encoded blocks, 30+ days):
```sql
CREATE TABLE telemetry_cold (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL,
    metric TEXT NOT NULL,
    block_start INTEGER NOT NULL,  -- Unix ms, aligned to hour boundary
    encoding TEXT NOT NULL,         -- 'gorilla' | 'rle'
    data_type TEXT NOT NULL,        -- 'analog' | 'digital'
    data BLOB NOT NULL,             -- Gorilla/RLE encoded binary
    count INTEGER NOT NULL,
    first_ts INTEGER NOT NULL,
    last_ts INTEGER NOT NULL
);
CREATE INDEX idx_cold_time ON telemetry_cold(device_id, metric, block_start);
```

**`deadband_config`** (per-tag deadband settings):
```sql
CREATE TABLE deadband_config (
    device_id TEXT NOT NULL,
    metric TEXT NOT NULL,
    deadband_type TEXT NOT NULL DEFAULT 'percent',  -- 'percent' | 'absolute'
    deadband_value REAL NOT NULL DEFAULT 0.1,
    PRIMARY KEY (device_id, metric)
);
```

### Deadband Logic

- **Analog tags** (default): 0.1% of current value. Only write if `|new_value - last_value| > max(deadband_value * |last_value|, absolute_min)`. The `absolute_min` prevents division-by-zero edge cases when value is near zero; default 0.001.
- **Digital tags** (default): 0 (store every change). Only write if `new_value != last_value`.
- Configurable per-tag via `deadband_config` table.
- Deadband check happens in the persistence task, before INSERT.
- Last-written value per `(device_id, metric)` tracked in memory for deadband comparison.

---

## Section 2: Compression Workers

Three background tokio tasks spawned at startup alongside existing persistence and retention tasks.

### Hot→Warm Worker (runs every 1 hour)

1. Query `telemetry_hot` for rows older than 24 hours
2. Group by `(device_id, metric, bucket_start)` where `bucket_start = floor(timestamp / 60000) * 60000`
3. For each bucket: compute OHLC (open/high/low/close), count, and varint-encode the delta sequence `(value - open)`
4. INSERT into `telemetry_warm`, DELETE from `telemetry_hot` — in a single transaction
5. Track compression ratio per run for stats reporting

### Warm→Cold Worker (runs every 24 hours)

1. Query `telemetry_warm` for rows older than 30 days
2. Group by `(device_id, metric, block_start)` where `block_start = floor(bucket_start / 3600000) * 3600000`
3. For analog metrics: apply Gorilla delta-of-delta encoding (bit-level per the paper)
4. For digital metrics: apply RLE encoding `(value, count)`
5. INSERT into `telemetry_cold`, DELETE from `telemetry_warm` — in a single transaction

### Archive Worker (on-demand via CLI)

1. Query `telemetry_cold` for blocks older than 1 year
2. Export to Parquet file with columnar layout
3. Optional: DELETE archived blocks from SQLite (default: delete; `--keep` flag to preserve)

### Gorilla Encoding (Paper-Accurate)

Following the Facebook Gorilla paper:
- **Timestamp encoding:** Delta-of-delta with variable-length bit packing. First value stored as-is, second stores delta, subsequent values store delta-of-delta with Huffman-like prefix codes.
- **Value encoding:** XOR with previous value. Leading zeros and trailing zeros stored as prefix codes. Only the meaningful bits are stored.
- **Block format:** Header (start_ts, count, first_value) + bitstream

### RLE for Digital Tags

- `(value: bool, count: u32)` pairs
- A pump running for 8 hours at 1Hz = 1 row instead of 28,800

### Error Handling

- Compression failures log errors but never crash the system
- Failed compression runs retry on next schedule
- Original data is only deleted after successful compression INSERT (transactional)
- Compression stats track success/failure counts

---

## Section 3: Decompression & Query Path

### Unified Query Function

A new `query_compressed()` function replaces `query_history()`. Logic:

1. Given a time range `[start, end]` and `(device_id, metric)`
2. Determine which tiers overlap:
   - Hot: last 24h
   - Warm: 1-30 days
   - Cold: 30+ days
3. Query each overlapping tier:
   - **Hot:** Direct SELECT, no decompression
   - **Warm:** SELECT OHLC buckets (aggregate mode) or reconstruct from deltas (detail mode)
   - **Cold:** SELECT + Gorilla/RLE decode to reconstruct individual data points
4. Merge results chronologically
5. Return unified `Vec<(timestamp, value)>` — same format as current API

### Transparency

- API endpoints (`/api/history`, `/api/data`) return the same JSON format
- Frontend requires zero changes for historical data queries
- `query_history()` signature unchanged, only internal implementation changes

### Warm Tier Query Modes

- **Aggregate mode** (default for chart display): Return OHLC buckets directly — no delta reconstruction, fast
- **Detail mode** (for export/CSV): Reconstruct individual points from varint deltas

---

## Section 4: Performance Optimizations

### SQLite WAL Mode Tuning

Enable at startup:
```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;     -- safe with WAL, faster than FULL
PRAGMA wal_autocheckpoint=1000; -- checkpoint every 1000 pages
PRAGMA cache_size=-32000;       -- 32MB page cache
PRAGMA temp_store=MEMORY;       -- temp tables in RAM
```

### Connection Pooling for Modbus TCP

- Maintain `HashMap<DeviceId, ModbusConnection>` pool
- Reuse TCP connections across polling cycles instead of connect/read/close each cycle
- Auto-reconnect on connection drop with exponential backoff
- Connection timeout: 5s, read timeout: 3s
- Max idle time: 60s (close connections unused for >60s)
- Thread-safe access via `Arc<Mutex<>>` or `Arc<RwLock<>>`

### Flash/SD Card Write Optimization

- Configurable persistence interval (default 10s for edge, 5s for server)
- Batch all writes into a single transaction per persistence cycle
- Use WAL autocheckpoint batching to reduce write amplification
- `voltrus --edge-mode` CLI startup flag for conservative defaults (persistence interval 10s, WAL checkpoint 2000, larger batch sizes). Also configurable in `config.yaml` under `edge_mode: true`.

### Memory Profiling

- `GET /api/v1/system/memory` endpoint reporting:
  - RSS (resident set size)
  - In-memory cache size
  - Compression buffer sizes
  - Connection pool size
  - WAL size
- Alert in system view if memory exceeds threshold (default 50MB idle)
- Stats tracked in `CompressionStats` struct updated by workers

### Performance Budget

| Metric | Budget | Alert Threshold |
|--------|--------|-----------------|
| Binary size | <30MB (release) | Build script warning |
| RAM (idle) | <50MB | System dashboard alert |
| Write throughput | 10K tags @ 1Hz on 1 vCPU/512MB | Benchmark test |
| Query latency (1h range) | <100ms | API log warning |
| Query latency (24h range) | <500ms | API log warning |

---

## Section 5: CLI Commands & API Endpoints

### CLI Commands

| Command | Description |
|---------|-------------|
| `voltrus db stats` | Storage stats: row counts per tier, DB size, compression ratio |
| `voltrus db compress --now` | Force-run all compression workers immediately |
| `voltrus db compress --tier warm` | Force-run specific tier compression |
| `voltrus archive` | Export cold data (>1 year) to Parquet/CSV |
| `voltrus archive --keep` | Export without deleting from SQLite |

### API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/compression/stats` | Compression statistics: tier sizes, ratios, last run timestamps |
| `GET /api/v1/compression/config` | Get compression config (retention periods, schedules) |
| `PUT /api/v1/compression/config` | Update compression config |
| `GET /api/v1/system/memory` | Memory usage stats (RSS, cache sizes, connection pool) |

### Frontend: Compression Stats Dashboard

New panel in system view with:
- **Storage overview:** Total DB size, per-tier row counts and sizes
- **Compression ratio:** Hot→Warm ratio, Warm→Cold ratio, overall ratio
- **Tier timeline:** Visual breakdown of data age distribution
- **Health indicators:** Last successful compression run, error counts
- **Deadband config:** Per-tag deadband settings (view/edit)

---

## Section 6: New Files & Module Structure

### Backend (Rust)

| File | Purpose |
|------|---------|
| `src/compression.rs` | Gorilla encoder/decoder, RLE encoder/decoder, varint encoding, BitWriter/BitReader |
| `src/deadband.rs` | Deadband filtering logic, in-memory last-value tracker, per-tag config |
| `src/api/compression.rs` | Compression stats/config API handlers |
| Modifications to `src/storage.rs` | New table creation, compression workers, unified query function, migration from old tables |
| Modifications to `src/main.rs` | Compression worker spawning, CLI command handling, edge-mode flag |
| Modifications to `src/modbus.rs` | Connection pooling, TCP connection reuse |
| Modifications to `src/api/mod.rs` | Register compression API routes |

### Frontend (React/TypeScript)

| Component | Purpose |
|-----------|---------|
| `client/src/components/CompressionStats.tsx` | Main compression stats dashboard panel |
| `client/src/components/DeadbandConfig.tsx` | Per-tag deadband configuration editor |
| `client/src/components/StorageOverview.tsx` | Storage usage breakdown visualization |

---

## Section 7: Migration Strategy

### storage.rs Merge Coordination with v0.24.0

v0.24.0 (Command Ack, other agent) adds a `commands` table to `storage.rs`. v0.25.0 adds `telemetry_hot`, `telemetry_warm`, `telemetry_cold`, `deadband_config` tables. Both append new code at the end of `storage.rs`.

**Merge plan:**
1. v0.24.0 merges first (earlier version)
2. v0.25.0 rebases onto merged result
3. Conflicts in `storage.rs` are additive (both append new tables/functions) — easy manual resolution

### Data Migration

On startup, if `telemetry_hot` doesn't exist but old `telemetry` does:
1. Rename `telemetry` → `telemetry_hot`
2. Rename `telemetry_hourly` → `telemetry_hourly_backup` (keep for safety)
3. Rename `telemetry_daily` → `telemetry_daily_backup` (keep for safety)
4. Create new tables (`telemetry_warm`, `telemetry_cold`, `deadband_config`)
5. Old hourly/daily backup tables dropped after first successful compression run

---

## Compression Storage Estimates (10K tags @ 1Hz)

| Tier | Compressed Size | Retention |
|------|----------------|-----------|
| Hot (raw + deadband) | ~12 GB/day → ~2.4 GB/day with deadband | 1 day |
| Warm (OHLC + deltas) | ~240 MB/day | 30 days (~7 GB) |
| Cold (Gorilla/RLE) | ~24 MB/day | 1 year (~8.6 GB) |

**Target: 1 year in ~15 GB total.**

---

## Testing Strategy

- **Unit tests:** Gorilla encoder/decoder roundtrip, RLE encoder/decoder roundtrip, deadband logic, varint encoding
- **Integration tests:** Full hot→warm→cold pipeline, query across all tiers, migration from old tables
- **Benchmarks:** 10K tags @ 1Hz write throughput, query latency at various time ranges, compression ratio on synthetic SCADA data
- **Edge case tests:** Empty buckets, single-point buckets, NaN/inf values, concurrent compression and query
