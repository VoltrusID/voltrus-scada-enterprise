# v0.26.0 — Sandboxed Scripting & Simulation Mode

**Date:** 2026-05-03
**Status:** Approved
**Version:** 0.26.0
**Timeline:** 6-8 weeks
**Tier:** Professional ($499, simulation + rollback) / Enterprise ($999, +sandboxed scripting)

---

## Goal

"Configure without touching live, then go live with one button." Business logic without risk. Three subsystems: simulation mode, sandboxed scripting, and config versioning with rollback.

---

## Architecture: Layered Integration

All three subsystems integrate into existing Voltrus infrastructure.

| Subsystem | Integration Point |
|-----------|------------------|
| **Simulation** | Per-device `simulation_mode` flag. PollerManager checks flag — if sim, skip real poll, inject fake data into cache. |
| **Scripting** | New `ScriptNode` in Flow Engine. JS compiled to WASM server-side, executed in wasmtime sandbox at trigger time. |
| **Config Versioning** | `config_snapshots` table. Every write to versioned tables auto-creates a snapshot before the change. |

**Single new dependency:** `wasmtime` crate (Bytecode Alliance WASM runtime for Rust). For JS→WASM: embed a pre-compiled QuickJS-WASM binary. User JS is evaluated inside QuickJS running inside wasmtime — double sandbox (JS sandbox inside WASM sandbox). No external toolchain or build step required from the engineer.

---

## Data Model

### New table: `sim_profiles`

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto ID |
| name | TEXT UNIQUE | e.g. "3-phase energy meter" |
| data_type | TEXT | `sinusoidal`, `random_walk`, `step`, `constant`, `square_wave` |
| config | TEXT | JSON config for the generator (amplitude, center, noise, period) |
| created_at | INTEGER | Timestamp |

### New table: `scripts`

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto ID |
| name | TEXT UNIQUE | Script name |
| source | TEXT | JavaScript source code |
| syntax_error | TEXT | Last syntax validation error if any, NULL if clean |
| cpu_limit_ms | INTEGER | Max execution time per invocation (default 1000) |
| memory_limit_bytes | INTEGER | Max WASM memory (default 1MB) |
| enabled | BOOLEAN | Whether script can execute |
| created_at | INTEGER | Created timestamp |
| updated_at | INTEGER | Last modified timestamp |

Note: No per-script WASM compilation. A single pre-compiled QuickJS-WASM interpreter binary is embedded in the server. Scripts are evaluated inside this interpreter at runtime.

### New table: `script_executions`

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto ID |
| script_id | INTEGER | FK to scripts |
| flow_id | TEXT | Which flow triggered it |
| trigger_type | TEXT | timer / tag_change / mqtt |
| duration_ms | INTEGER | Execution time |
| status | TEXT | success / error / timeout / oom |
| error_message | TEXT | Error if failed |
| output | TEXT | JSON output if succeeded |
| created_at | INTEGER | Timestamp |

### New table: `config_snapshots`

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER PK | Auto ID (monotonic — higher = newer) |
| label | TEXT | Optional human label |
| description | TEXT | Auto-generated description of what changed |
| snapshot_data | TEXT | JSON blob of all versioned config tables |
| trigger | TEXT | `auto` / `manual` / `restore` / `factory_reset` |
| created_by | TEXT | User who triggered the change |
| created_at | INTEGER | Timestamp |

Indexes: `config_snapshots(created_at)`, `config_snapshots(trigger)`.

### Modifications to `devices` table

| Column | Type | Default | Description |
|--------|------|---------|-------------|
| simulation_mode | BOOLEAN | 0 | Whether this device is in simulation mode |
| sim_profile_id | INTEGER | NULL | FK to sim_profiles |
| sim_config | TEXT | NULL | Per-device override JSON for sim profile params |

### Modifications to `data_sources` table

| Column | Type | Default | Description |
|--------|------|---------|-------------|
| sim_value | REAL | NULL | For constant sim mode — fixed value override |

---

## Simulation Mode

### How it works

When a device has `simulation_mode = 1`:

1. PollerManager checks the flag before each poll cycle
2. If sim=true, skip the real protocol read entirely
3. Generate simulated value based on `sim_profile` + per-device `sim_config` overrides
4. Inject the simulated value into the **same cache** as real data
5. Everything downstream (SSE, flows, alarms, scripts) works identically

### Sim generators

| Generator | Parameters | Output |
|-----------|------------|--------|
| `sinusoidal` | amplitude, center, period_sec, noise_pct | Smooth sine wave with optional noise |
| `random_walk` | start_value, step_size, min, max | Brownian motion within bounds |
| `step` | low_value, high_value, step_period_sec | Alternates between two values |
| `constant` | value | Fixed value (set per data_source via sim_value or sim_config) |
| `square_wave` | high_value, low_value, period_sec, duty_pct | Square wave with duty cycle |

### Go-live button

When switching a device from sim → live:

1. Auto-create a config snapshot (labeled "pre-go-live: {device_name}")
2. Toggle `simulation_mode = 0` on the device
3. Next poll cycle connects to real PLC
4. If the first real poll fails within 30s, **auto-revert to sim** with a "go-live failed" notification
5. UI shows a confirmation dialog: "Switching to LIVE mode. Device will connect to real PLC at {ip}:{port}."

### UI indicators

- Device card shows yellow **SIM** badge when simulation is active
- Data points from simulated devices carry a `simulated: true` flag in SSE payloads
- Trend charts optionally show dotted lines for simulated data vs solid for live

---

## Sandboxed Scripting Engine

### Architecture

```
Engineer writes JS in web UI editor
         │
         ▼
   POST /api/v1/scripts  (save + compile)
         │
         ▼
   JS → evaluated inside QuickJS-WASM (pre-compiled QuickJS binary running in wasmtime)
         │
         ├─ Success → store source, clear syntax_error
         └─ Failure → store syntax_error
         │
         ▼
   Flow Engine triggers (timer / tag change / MQTT)
         │
         ▼
   ScriptNode in flow → loads source from DB
         │
         ▼
   wasmtime sandbox execution
   - CPU timeout: cpu_limit_ms (default 1000ms)
   - Memory cap: memory_limit_bytes (default 1MB)
   - No filesystem, no network, no random, no clock
         │
         ▼
   Output → downstream flow nodes (same as any node)
```

### Script API

```js
function main(input) {
  const voltage = input.tags["PLC_01.voltage"];
  const current = input.tags["PLC_01.current"];
  const power = voltage * current;

  return {
    outputs: { power: power },
    alarms: [{ severity: "warning", message: `High power: ${power}` }]
  };
}
```

**Scripts receive** (read-only):
- `input.tags` — current cached values for all tags (keyed by `device.tag`)
- `input.prev` — previous tag values (for change detection)
- `input.trigger` — which trigger fired (`"timer"`, `"tag_change"`, `"mqtt"`)
- `input.metadata` — flow name, timestamp

**Scripts return**:
- `outputs` — key-value pairs passed to downstream flow nodes
- `alarms` — optional alarm objects to generate

**Scripts CANNOT**:
- Write to devices (no command queue access)
- Access filesystem, network, environment variables
- Import modules or load external code
- Loop infinitely (CPU timeout kills execution)
- Access more than memory_limit_bytes of RAM

### Flow Engine integration

New `ScriptNode` variant:

```rust
ScriptNode {
    script_id: i64,
    inputs: Vec<String>,  // tag keys to pass to script
}
```

Positioned in flow graph like any other node. Upstream nodes feed into it, its outputs feed downstream.

### Error handling

- Script crashes → log error to `script_executions`, continue flow with empty output
- Script times out → kill WASM instance, log timeout error
- Script uses too much memory → OOM trap, log memory error
- Never crashes the main process, never blocks other flows

---

## Config Versioning & Rollback

### What gets versioned

Every write mutation to these tables auto-creates a snapshot before the change:

| Table | What's captured |
|-------|----------------|
| `devices` | Device configs (IP, port, protocol, registers, simulation settings) |
| `data_sources` | Tag definitions (name, type, alarms, write policies, sim overrides) |
| `flows` | Flow definitions (nodes, connections, triggers) |
| `alarms` | Alarm rules and thresholds |
| `screens` | Dashboard/screen layouts |
| `recipes` | Recipe parameter sets |

**Not versioned** (ephemeral): telemetry, commands, script_executions, security_events, audit_log.

### Snapshot lifecycle

```
Engineer changes device config via API
         │
         ▼
   Before write: generate description of change
         │
         ▼
   SELECT all versioned tables → serialize to JSON → INSERT config_snapshot
         │
         ▼
   Apply the actual change
```

Description auto-generated from the API call: e.g. "Updated device PLC_01: changed simulation_mode 0→1, sim_profile_id null→3"

### Rollback

`POST /api/v1/config/snapshots/{id}/rollback`

1. Load snapshot JSON
2. Diff current state vs snapshot (show what will change)
3. Confirm dialog in UI
4. Begin transaction: delete current rows in versioned tables, insert snapshot rows
5. Commit
6. Auto-create a new snapshot labeled "rollback to #{id}"

### Diff viewer

`GET /api/v1/config/snapshots/{from_id}/diff/{to_id}`

Returns a structured diff:
```json
{
  "devices": {
    "added": [],
    "removed": [{"id": "PLC_03", "name": "Backup pump"}],
    "changed": [{"id": "PLC_01", "field": "simulation_mode", "from": 0, "to": 1}]
  },
  "flows": { ... }
}
```

### CLI commands

| Command | Description |
|---------|-------------|
| `voltrus config list` | Show recent snapshots |
| `voltrus config rollback --to 45` | Rollback to snapshot #45 |
| `voltrus factory-reset` | Wipe DB to fresh state (creates final snapshot first) |
| `voltrus restore --from-backup` | Already exists — unchanged |

### Retention

Keep last 50 snapshots. On each new snapshot, delete oldest if count > 50. Manual snapshots are never auto-deleted.

---

## API Endpoints

### Simulation

| Method | Path | Permission | Description |
|--------|------|------------|-------------|
| GET | `/api/v1/sim-profiles` | `sim:read` | List sim profiles |
| POST | `/api/v1/sim-profiles` | `sim:write` | Create sim profile |
| PUT | `/api/v1/sim-profiles/{id}` | `sim:write` | Update sim profile |
| DELETE | `/api/v1/sim-profiles/{id}` | `sim:write` | Delete sim profile |
| POST | `/api/v1/devices/{id}/simulate` | `sim:write` | Toggle sim mode on device |
| POST | `/api/v1/devices/{id}/go-live` | `sim:write` | Switch device to live (with auto-revert) |

### Scripts

| Method | Path | Permission | Description |
|--------|------|------------|-------------|
| GET | `/api/v1/scripts` | `scripts:read` | List scripts |
| POST | `/api/v1/scripts` | `scripts:write` | Create script (validates + saves) |
| PUT | `/api/v1/scripts/{id}` | `scripts:write` | Update script (re-validates) |
| DELETE | `/api/v1/scripts/{id}` | `scripts:write` | Delete script |
| POST | `/api/v1/scripts/{id}/test` | `scripts:write` | Test-run script with sample input |
| GET | `/api/v1/scripts/{id}/executions` | `scripts:read` | Execution history for a script |

### Config Snapshots

| Method | Path | Permission | Description |
|--------|------|------------|-------------|
| GET | `/api/v1/config/snapshots` | `config:read` | List snapshots |
| GET | `/api/v1/config/snapshots/{id}` | `config:read` | Get snapshot detail |
| POST | `/api/v1/config/snapshots` | `config:write` | Create manual snapshot |
| GET | `/api/v1/config/snapshots/{from}/diff/{to}` | `config:read` | Diff two snapshots |
| POST | `/api/v1/config/snapshots/{id}/rollback` | `config:write` | Rollback to snapshot |

### Permissions

| Permission | Description |
|------------|-------------|
| `sim:read` | View simulation profiles and device sim status |
| `sim:write` | Toggle simulation, create/edit profiles |
| `scripts:read` | View scripts and execution history |
| `scripts:write` | Create/edit/delete/test scripts |
| `config:read` | View config snapshots |
| `config:write` | Create manual snapshots, rollback |

Updates `ALL_PERMISSIONS` from 18 → 24. `OPERATOR_PERMISSIONS` gets `sim:read`, `scripts:read`, `config:read`.

### SSE events

| Event | Payload | When |
|-------|---------|------|
| `device_sim_status` | `{device_id, simulation_mode, sim_profile}` | Device sim toggle |
| `script_compiled` | `{script_id, name, success, error?}` | Script saved/compiled |
| `config_snapshot` | `{snapshot_id, description, trigger}` | Config snapshot created |

---

## Frontend

### Simulation controls

- **Device settings panel**: "Simulation" section with toggle switch, sim profile dropdown, per-tag value overrides
- **SIM badge**: Yellow badge on device cards/tags when simulation is active
- **Go-live dialog**: Confirmation modal showing device IP/port, with auto-revert countdown if first poll fails
- **Sim profile manager**: CRUD for sim profiles with live preview of the generator waveform

### Script editor

- **Monaco editor** (same as VS Code): syntax highlighting, line numbers, error markers
- **Test panel**: Side panel with sample input JSON, "Run Test" button, output/error display
- **Execution history**: Table of recent runs with status, duration, expandable output

### Config snapshots panel

- **Snapshot list**: Chronological list with trigger badge (auto/manual/restore), description, creator
- **Diff viewer**: Side-by-side comparison of two snapshots with color-coded additions/removals/changes
- **Rollback button**: On any snapshot, with confirmation dialog showing diff of what will change

### Integration points

| Existing Component | Addition |
|--------------------|----------|
| Device settings | Simulation section (toggle, profile, overrides) |
| Device cards/grid | SIM badge when simulated |
| Trend charts | Dotted lines for simulated data points |
| Flow editor | "Script" node type in node palette |
| Header | "Snapshots" button → config snapshot panel |

---

## Files to Modify/Create

### Rust backend

| File | Change |
|------|--------|
| `Cargo.toml` | Add `wasmtime` dependency |
| `storage.rs` | Add `sim_profiles`, `scripts`, `script_executions`, `config_snapshots` tables. Add columns to `devices` and `data_sources`. New CRUD methods for all. |
| `models.rs` | `SimProfile`, `Script`, `ScriptExecution`, `ConfigSnapshot`, `ConfigDiff` structs. New permission constants. |
| `api/simulation.rs` | New file — sim profile CRUD + device simulate/go-live handlers |
| `api/scripts.rs` | New file — script CRUD, compile, test, execution history handlers |
| `api/config_snapshots.rs` | New file — snapshot list, detail, manual create, diff, rollback handlers |
| `api/mod.rs` | Register new route modules |
| `sim_engine.rs` | New file — sim value generators (sinusoidal, random_walk, step, constant, square_wave) |
| `script_engine.rs` | New file — QuickJS-WASM interpreter setup, wasmtime sandbox execution, resource limits, syntax validation |
| `poller_manager.rs` | Check `simulation_mode` flag before poll, call sim_engine for fake data |
| `flow_engine/node.rs` | Add `ScriptNode` variant |
| `flow_engine/engine.rs` | Handle `ScriptNode` execution (load WASM, run in sandbox, pass output downstream) |
| `cache.rs` | Add `simulated` flag to cached metric entries |
| `main.rs` | Add new modules, CLI subcommands for config/factory-reset |
| `cli.rs` or `main.rs` | Add `voltrus config list`, `voltrus config rollback --to N`, `voltrus factory-reset` |

### Frontend

| File | Change |
|------|--------|
| `client/src/api/simulation.ts` | New file — sim profile + device sim API client |
| `client/src/api/scripts.ts` | New file — script API client |
| `client/src/api/config-snapshots.ts` | New file — config snapshot API client |
| `client/src/components/SimProfileManager.tsx` | New file — sim profile CRUD with waveform preview |
| `client/src/components/SimControls.tsx` | New file — device simulation toggle + go-live dialog |
| `client/src/components/ScriptEditor.tsx` | New file — Monaco-based script editor with test panel |
| `client/src/components/ScriptExecutionHistory.tsx` | New file — execution history table |
| `client/src/components/ConfigSnapshotPanel.tsx` | New file — snapshot list, diff viewer, rollback |
| `client/src/views/DeviceView.tsx` | Add SIM badge, simulation section in device settings |
| `client/src/views/FlowEditor.tsx` | Add "Script" node type to palette |
| `client/src/App.tsx` | Add "Snapshots" button to header |
| `client/src/TrendChart.tsx` | Dotted lines for simulated data points |

---

## Testing Strategy

- Unit tests for each sim generator (sinusoidal, random_walk, step, constant, square_wave)
- Unit tests for JS syntax validation success and failure cases
- Unit tests for wasmtime sandbox execution with resource limits (CPU timeout, memory cap)
- Unit tests for script API (input parsing, output validation, error handling)
- Unit tests for config snapshot creation and rollback
- Unit tests for config diff generation
- Integration tests for full sim lifecycle: create profile → toggle sim → verify fake data in cache → go-live → verify real data
- Integration tests for script in flow: create script → add ScriptNode to flow → trigger → verify output flows downstream
- Integration tests for config rollback: make changes → snapshot → make more changes → rollback → verify original state restored
- API endpoint tests matching existing pattern in `src/api/`
