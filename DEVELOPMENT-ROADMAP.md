# Voltrus — Post-Launch Roadmap

*Last updated: May 2026 — v0.39.0 shipped, all items through v0.39.0 complete*

---

## Project Structure

```
voltrus/                          # Monorepo root
├── voltrus/                      # Main application (Rust backend + React frontend)
│   ├── src/                      # Rust backend (Axum web server, protocol drivers, storage)
│   │   ├── main.rs               # Binary entry point
│   │   ├── api/                  # REST API handlers
│   │   ├── auth/                 # Auth (local Argon2, OIDC, SAML, LDAP)
│   │   ├── license/              # License validation (Ed25519, tier enforcement)
│   │   ├── notification/         # Email, webhook, push, SMS
│   │   ├── integration/          # Prometheus exporter, BI connectors, data egress
│   │   ├── flow_engine/          # Data Flow Engine (node-based automation)
│   │   ├── modbus.rs             # Modbus TCP driver
│   │   ├── opcua.rs              # OPC-UA client
│   │   ├── s7.rs                 # Siemens S7 driver
│   │   ├── ethernet_ip.rs        # Allen-Bradley EtherNet/IP driver
│   │   ├── storage.rs            # SQLite storage layer
│   │   ├── security.rs           # IEC 62443 security audit mode
│   │   └── ...                   # analytics, compliance, hub, oee, recipes, etc.
│   ├── client-lit/              # Lit frontend (TypeScript + Vite)
│   │   └── src/                  # Components, views, panels, services
│   ├── static/                   # Static assets served by Rust
│   ├── config.yaml               # Runtime configuration
│   └── Cargo.toml                # Rust dependencies
│
├── voltrus-landing/              # Marketing site (static HTML, deployed to Cloudflare Pages)
│   ├── index.html                # Landing page with pricing, features, download
│   ├── docs.html                 # Documentation
│   ├── changelog.html            # Release notes
│   ├── activate.html             # License activation page
│   └── blog/                     # Blog posts
│
└── voltrus-cloudflare-worker/    # License activation server (Cloudflare Workers)
    └── src/index.ts              # Handles activation requests, tier validation
```

**Development is fullstack** — each feature touches both Rust backend (`voltrus/src/`) and Lit frontend (`voltrus/client-lit/src/`). The landing page (`voltrus-landing/`) must be updated per release: changelog, docs, and feature list.

---

## Design Principles

1. **No AI in the control loop** — r/PLC overwhelming rejection of autonomous AI in OT. AI must never write commands to equipment, fire alarms autonomously, or operate in the data collection layer. AI is acceptable in the engineering/configuration layer only (tag mapping, layout generation, alarm setup) where the engineer reviews and approves before deployment. See v0.26.5 for AI-assisted engineering.
2. **Own the protocol stack** — never build on Kepware, Matrikon, or middleware wrappers. Native Rust drivers only.
3. **Single binary, no external dependencies** — SQLite, not Postgres. Embedded, not Docker. Bare-metal first.
4. **SCADA IT ≠ corporate IT** — separate RBAC, local auth by default. Don't over-integrate with corporate AD.
5. **Passive by default** — poll/read-only monitoring. Write commands are explicit opt-in per device.

---

## Current State (v0.39.0)

All features from v0.18.0 through v0.39.0 have been implemented and shipped. The current binary includes:

### Protocols
- Modbus TCP (all tiers)
- MQTT (all tiers)
- OPC-UA (Professional+)
- Siemens S7 (Enterprise)
- Allen-Bradley EtherNet/IP (Enterprise)
- DNP3 (Enterprise)
- BACnet/IP (Enterprise)

### Dashboards & Visualization
- Grid view, Canvas view, 3D Digital Twin, 2D SCADA editor
- Interactive trend charts with pan/zoom, multi-pen overlay, square-wave rendering
- Circular/bar gauges, LED indicators
- Virtualized data tables (10K rows at 60fps)
- Touch-first design (48px min targets)
- PWA manifest for mobile

### Notifications
- Email via SMTP (Starter+)
- Webhook via Data Flow Engine (Starter+)
- Web Push with VAPID (Starter+)
- SMS via email-to-SMS gateways (Professional+)

### Data & Operations
- REST API with API key auth, CORS, query params
- Backup/restore CLI
- Git-friendly config export/import/validate
- Modbus TCP auto-discovery (subnet scan)
- OEE / downtime tracking with shift reporting
- Data Flow Engine (8 node types, 2 trigger types)
- Smart tiered data retention
- Data compression (Gorilla delta-of-delta, RLE, deadband filtering)
- WebSocket bidirectional streaming with MessagePack
- OpenAPI 3.1 spec with auto-generated Python/TypeScript SDKs
- Sandboxed scripting (WASM) with simulation/go-live mode
- AI-assisted engineering copilot (BYOK + Voltrus AI hosted)
- Advanced analytics engine (SPC, histograms, run charts, custom metrics)

### Multi-Site & Reporting
- Hub dashboard aggregating multiple sites
- Automated PDF reporting with scheduled email
- Recipe / parameter management

### OT Asset Inventory
- Auto-populated device inventory (PLC, RTU, HMI, sensors)
- Firmware version tracking and update flagging
- IP address inventory with conflict detection
- Maintenance history log with technician and notes
- SNMP scanning for network-visible assets
- NIS2 compliance export with risk scoring
- Asset tags (location, vendor, criticality, type)
- CSV/XLSX import for existing asset spreadsheets

### EDR Compatibility & OT Deployment
- Automatic EDR/antivirus detection (Defender, Trellix, CrowdStrike, SentinelOne, etc.)
- Recommended exclusion paths per installation
- File I/O optimization (batch writes, WAL mode, sequential writes)
- PowerShell script generation for Defender exclusions
- Group Policy deployment guide for IT teams
- CLI: `voltrus edr report`, `voltrus edr exclusions`, `voltrus edr exclusions-ps1`
- API: `/api/v1/edr/report`, `/api/v1/edr/exclusions`, `/api/v1/edr/io-status`

### Auth & Compliance
- Local auth (Argon2)
- RBAC (Admin, Operator, Viewer, Custom) with audit trail
- SSO / OIDC (Keycloak, Auth0, Azure AD)
- SAML 2.0 (ADFS)
- LDAP (legacy Active Directory)
- FDA 21 CFR Part 11 compliance mode
- RTSP video camera integration

### Hardware Certification & Edge Deployment
- Platform detection (CPU, RAM, storage type, board model)
- Hardware compatibility matrix (10 certified platforms)
- Performance benchmark engine (tag write, batch, query, compression)
- Edge deployment guides (Raspberry Pi, Advantech, Siemens IPC, generic PC)
- SD card write optimization verification
- CLI: `voltrus hw report`, `voltrus hw benchmark`, `voltrus hw edge-deploy`, `voltrus hw compat-matrix`
- API: `/api/v1/hw/report`, `/api/v1/hw/benchmark`, `/api/v1/hw/compatibility-matrix`, `/api/v1/hw/edge-deploy`

### Licensing
- 4-tier system: Free Trial (30 days), Starter ($249), Professional ($499), Enterprise ($999)
- Polar.sh integration with Ed25519 device-bound activation
- Air-gap friendly (offline activation flow)
- Cloudflare worker for license server

### External Integration & BI Connectors
- Prometheus exporter endpoint (`/metrics`) with tag values, device status, alarm counts
- REST API BI query endpoint with filtering, pagination, CSV/JSON output (Power BI, Metabase)
- Grafana-compatible JSON endpoint with metric search and `[value, timestamp]` datapoints
- Outbound webhooks for alarm and tag_change events with retry logic and device filtering
- Scheduled data egress: CSV/Parquet export to filesystem or S3-compatible storage (hourly/daily/weekly/custom)
- Connection info endpoint with auto-generated connection strings for Grafana, Power BI, Metabase
- CLI: `voltrus egress list`, `voltrus egress run <ID>`
- API: `GET /metrics`, `GET /api/v1/integration/bi/query`, `GET /api/v1/integration/grafana/query`, `GET /api/v1/egress/jobs`, `GET /api/v1/integration/webhooks/outbound`, `GET /api/v1/integration/connection-info`

---

## Post-Launch Roadmap

*Reshuffled May 2026 — all items reprioritized based on competitive analysis (LiRAYS-SCADA) and SCADA industry veteran feedback. v0.34.0 shipped out of order before v0.32.0.*

### v0.22.0 — IEC 62443 Security Audit Mode (COMPLETE)

**Goal:** Enterprise security compliance for critical infrastructure.

- ✅ Security event logging: failed logins, config changes, device connect/disconnect
- ✅ Access attempt tracking with source IP and user agent
- ✅ Security dashboard widget showing recent events (API endpoints)
- ✅ Configurable alert thresholds for suspicious activity
- ✅ Append-only security log table (separate from audit trail)
- ✅ Export security logs as CSV for external audit tools
- **Tier:** Enterprise ($999)
- **API:** `GET/PUT /api/v1/security/thresholds`, `GET /api/v1/security/events`, `GET /api/v1/security/events/export`

---

### v0.23.0 — Alarm Management Overhaul (COMPLETE)

**Goal:** Fix alarm fatigue — the #2 complaint in SCADA. Operators disabling alarms and missing critical events kills people.

- ✅ Non-repeating alarms: fire once, don't re-fire until acknowledged (no alarm floods)
- ✅ Alarm shelving: temporarily suppress an alarm with auto-restore timer and audit log
- ✅ Alarm priority tiers: critical / warning / info — with per-tier notification routing
- ✅ Alarm summary dashboard: active alarms, shelved alarms, recent history, all filterable
- ✅ Audit trail for every alarm state change (who dismissed, who shelved, when it auto-restored)
- ✅ Alarm analytics: most frequent alarms, average time-to-acknowledge, alarm flood detection
- ✅ Deadband on alarm points: don't re-alarm if value oscillates around threshold
- **Tier:** Starter ($249, basic) / Professional ($499, shelving + analytics)
- **API:** `POST /api/v1/alarms/{id}/shelf`, `POST /api/v1/alarms/{id}/unshelf`, `GET /api/v1/alarms/audit`, `GET /api/v1/alarms/analytics`, `GET /api/v1/alarms/summary`

---

### v0.24.0 — Annotations & Shift Handover (COMPLETE)

**Goal:** Operator collaboration features — notes, handover, mentions.

- ✅ Click any data point → add annotation (text + timestamp + author)
- ✅ Shift handover log: structured "what happened, what's pending, what to watch"
- ✅ Annotation markers on trend charts at annotated timestamps
- ✅ Auto-generated shift report: events, alarms, annotations
- ✅ @mention another user → notification on next login
- **Tier:** Professional ($499)
- **API:** `POST/GET/PUT/DELETE /api/v1/annotations`, `POST/GET /api/v1/shift-handover`, `GET /api/v1/shift-handover/auto-report`, `GET/PUT/DELETE /api/v1/user-notifications`

---

### v0.24.0 — Command Ack & Control Mode (COMPLETE)

**Goal:** Voltrus is SCADA, not just monitoring. Reliable bidirectional control with acknowledgement.

- ✅ Write with ack: `POST /api/v1/tags/{id}/write` returns `202 Accepted` with `command_id`
- ✅ Command status tracking: `GET /api/v1/commands/{command_id}` → pending/sent/acked/failed
- ✅ Real-time ack via SSE: new event type `command_ack` pushed to connected clients
- ✅ Command history API: `GET /api/v1/commands?tag=X&status=failed` for audit trail
- ✅ Offline command queue: queue writes, replay when device reconnects
- ✅ Command audit log entry (who wrote what, when, result)
- ✅ Write protection mode: passive by default (read-only), explicit opt-in to enable writes per device
- **Tier:** Starter ($249, basic write) / Professional ($499, full command tracking + queue)

---

### v0.25.0 — Data Compression & Performance (COMPLETE)

**Goal:** Years of data in <15GB, not <15GB/day. Merges original v0.27.0 perf work with compression.

- ✅ Deadband filtering on write path (per-tag configurable, default 0.1% analog, 0 for digital)
- ✅ Background compression worker: hot → warm (1-min delta-encoded aggregates)
- ✅ Cold compression worker: warm → cold (Gorilla-style delta-of-delta for numeric, RLE for digital)
- ✅ Decompression layer in query path (transparent to frontend)
- ✅ CLI commands: `voltrus db stats`, `voltrus db compress --now`, `voltrus archive`
- ✅ Benchmark: 10,000 tags at 1Hz on 1 vCPU / 512MB RAM
- ✅ Connection pooling for Modbus TCP (reuse TCP connections across devices)
- ✅ SQLite WAL mode tuning for concurrent read/write
- ✅ Memory profiling and leak detection under sustained load
- ✅ Performance budget: alert if binary >30MB, RAM >50MB idle
- ✅ Flash/SD card write optimization: batch writes to reduce write amplification on edge devices (Raspberry Pi)
- **Target:** 1 year of 10K-tag data in ~15GB
- **See:** Appendix A — Data Compression Strategy

---

### v0.26.0 — Sandboxed Scripting & Simulation Mode (COMPLETE)

**Goal:** "Configure without touching live, then go live with one button." Business logic without risk.

- ✅ Simulation/staging mode: run Voltrus with simulated device data, no real PLC connections
- ✅ "Go live" button: switch from simulation to production in one click
- ✅ One-button config rollback: version configs, instant revert on failed rollout
- ✅ Sandboxed scripting engine: WASM scripts with CPU/memory limits (wasmtime)
- ✅ Scripts triggered by tag events (e.g., "when pump starts, verify valve path is open")
- ✅ Script editor in web UI with syntax highlighting
- ✅ Script error handling: failed scripts log errors but never crash the system
- ✅ Config versioning: every config change creates a versioned snapshot
- ✅ `voltrus factory-reset` + `voltrus restore --from-backup` one-liner for disaster recovery
- **Tier:** Professional ($499, simulation + rollback) / Enterprise ($999, +sandboxed scripting)
- **Source:** SCADA veteran — "this is key for real deployments"

---

### v0.26.5 — AI-Assisted Engineering Copilot (COMPLETE)

**Goal:** AI helps the integrator *configure* faster. AI never touches live equipment. Engineer drafts, reviews, then deploys to simulation first.

**Core principle:** AI operates exclusively in the engineering/development layer. It generates configs, layouts, and mappings that the engineer reviews before deployment. Zero commands reach real equipment through AI. This is a copilot, not an autopilot.

- **Modbus register auto-mapping:** Upload device datasheet PDF or CSV → extract register addresses, data types, scaling factors → generate tag configs. Engineer reviews and edits before saving.
- **Dashboard layout generation:** Describe the plant in plain text ("6 pumps, 2 flow meters, 4 pressure sensors, 2 tanks") → auto-generate dashboard layout with grouped widgets, gauge placements, and trend chart selections. Engineer rearranges before deploying.
- **Alarm threshold suggestions:** Select device type (pump station, power meter, VFD) → suggest alarm thresholds based on ISA-18.2 defaults and common patterns. Engineer adjusts per-site requirements.
- **Tag naming batch conversion:** "Rename all tags to ISA-5.1 convention" or "convert from PUMP1_START to PMP001_RUN" → batch rename with diff preview before applying.
- **Data flow rule generation:** "When tank level exceeds 80%, stop pump 1 and send SMS alarm" → generate flow engine rule with all node connections. Engineer reviews the logic graph before enabling.
- **Equipment template wizard:** Select industry + equipment type → pre-populate tag list, alarms, dashboard layout, and data flow rules from vertical templates (see v0.37).
- **Simulation-first workflow:** All AI-generated configs deploy to simulation mode (v0.26) by default. Engineer validates with simulated data before switching to live.
- **Audit trail:** Every AI-generated config is tagged with `source: ai-assist` in the config version history. Full diff against manual configs.
- **Privacy:** No tag data, device IPs, or plant information sent to cloud by default. BYOK mode sends only the prompts (datasheet text, layout descriptions) — not live data. Voltrus AI hosted mode applies the same principle: prompts only, no runtime data.

**Why this doesn't violate the "No AI" design principle:**
The No AI principle (Design Principle #1) prohibits AI in the *control/operations* layer — no autonomous commands to real equipment. This feature uses AI in the *engineering/configuration* layer — the same way a linter or code formatter helps a developer. The engineer is always in the loop, reviews every output, and explicitly deploys. The AI never writes to a PLC, never changes a setpoint, never operates equipment.

| Layer | AI Allowed? | Why |
|-------|-------------|-----|
| Control/operations (writing to PLCs, alarms firing, commands) | **Never** | Safety-critical. Deterministic only. |
| Data collection (reading registers, storing values) | **Never** | Must be reliable and verifiable. |
| Engineering/configuration (tag mapping, dashboard layout, alarm setup) | **Yes — with review** | Human reviews before deployment. No live equipment touched. |

- **Tier:** Available on all paid tiers (Starter+). See AI Copilot pricing below.
- **Dependencies:** v0.26 (simulation mode for safe testing of AI-generated configs), v0.37 (vertical templates for equipment wizard)

#### AI Copilot Pricing Model

Core Voltrus remains lifetime license with no recurring fees. AI Copilot is an optional productivity add-on with three tiers:

| Tier | Price | How it works | Best for |
|------|-------|--------------|----------|
| **BYOK** | Free (bring your own key) | Engineer pastes their OpenAI/Anthropic API key into Voltrus settings. Pays their own API costs directly. | Engineers who already have API keys and don't mind managing billing |
| **Voltrus AI** | $19/month per engineer seat | Voltrus-hosted inference. Best models. Unlimited usage. 1 seat covers all deployments. | Engineers who want convenience — no setup, no API keys, best results |

**Key business rules:**
- 1 AI seat = 1 engineer, unlimited deployments. An integrator with 10 client sites pays $249×10 for Voltrus licenses but only 1×$19/month for AI.
- BYOK is always free — no paywall on AI features, only on hosted inference convenience.
- AI Copilot subscription is completely separate from the core Voltrus license. Engineers who cancel AI still have full Voltrus functionality.
- This preserves the "lifetime license, no recurring fees" brand promise for the core product. AI is positioned as an optional productivity booster, not a required add-on.
- Engineers who don't want AI lose zero functionality. All manual configuration workflows remain first-class.

**Revenue model:** If 10% of Voltrus licensees subscribe to AI Copilot at $19/month: 100 licenses sold/month → 10 AI subscribers → $1,900/month recurring. Covers inference costs (~$200-400/month at scale) with healthy margin.

---

### v0.27.0 — SDKs & OpenAPI (COMPLETE)

**Goal:** Match LiRAYS developer experience. Official SDKs, auto-generated from API spec.

- ✅ OpenAPI 3.1 spec for all `/api/v1/*` endpoints (generated via `utoipa`)
- ✅ Auto-generated Python SDK → publish to PyPI
- ✅ Auto-generated TypeScript SDK → publish to npm
- ✅ CI pipeline: on API spec change → regenerate + publish SDKs
- ✅ SDK example scripts: "Read tag values", "Write setpoint", "Subscribe to alarms"
- ✅ SDK documentation section on docs site
- ✅ Swagger UI wired into router, `--dump-openapi` CLI flag
- **Tier:** Free (SDKs are open-source, API requires any paid tier)
- **See:** Appendix B — API & SDK Strategy

---

### v0.28.0 — Advanced Analytics Engine (COMPLETE)

**Goal:** Replace Python/Tableau exports for basic analysis.

- ✅ Time-series aggregation engine (min/max/avg/stddev/count per interval)
- ✅ Statistical widgets: run charts, histograms, control charts (SPC)
- ✅ Comparison views: this-shift vs last-shift, this-week vs last-week
- ✅ Custom calculation builder (define metrics from existing metrics)
- ✅ Scheduled analytics reports (daily/weekly email digest)
- **Tier:** Starter ($249, basic) / Professional ($499, advanced)

---

### v0.29.0 — WebSocket Bidirectional Streaming (COMPLETE)

**Goal:** True bidirectional real-time for high-frequency control loops and state sync.

- ✅ WebSocket endpoint alongside REST+SSE (not replacing)
- ✅ Message types: `tag_write`, `tag_update`, `command_ack`, `device_status`, `alarm`
- ✅ MessagePack binary format for efficiency without Protobuf complexity
- ✅ Browser-native — no proxy required (unlike gRPC-Web)
- ✅ REST+JSON stays for CRUD/config/queries; WebSocket only for tag streams
- **Tier:** Starter ($249)
- **See:** Appendix B — API & SDK Strategy

---

### v0.30.0 — DNP3 Protocol Support (COMPLETE)

**Goal:** Open the electric utility vertical. DNP3 is the standard for power utilities.

- ✅ DNP3 outstation/client support (native Rust implementation)
- ✅ Support: binary output, analog input, counter input, frozen counters
- ✅ DNP3 device configuration in web UI alongside existing protocols
- ✅ Auto-discovery of DNP3 outstations on network
- ✅ Data-level and class-based polling (class 0/1/2/3)
- ✅ DNP3 outstation simulator binary for testing
- **Tier:** Enterprise ($999)

---

### v0.31.0 — BACnet Protocol Support (COMPLETE)

**Goal:** Building automation vertical — HVAC, lighting, access control.

- ✅ BACnet/IP protocol support
- ✅ Read/write: analog/binary/multistate values
- ✅ BACnet device discovery
- ✅ COV (Change of Value) subscriptions for efficient polling
- ✅ BACnet simulator binary for testing
- **Tier:** Enterprise ($999)

---

---

### v0.32.0 — Polish & Usability (COMPLETE)

**Goal:** Industrial-grade UX. Themes, i18n, tag browsing, data migration.

- ✅ Tag browser with UNS-style tree view (hierarchical device/tag navigation)
- ✅ Tag naming convention enforcement: built-in ISA-5.1 template, custom templates, validation on tag creation (no more "Pump1_Start" vs "PMP001_RUN" for the same thing)
- ✅ Dark/light theme toggle at runtime (no restart)
- ✅ Colorblind-friendly indicators: shape + color (circle=good, triangle=fault, diamond=warning), not color alone
- ✅ Multi-language UI: Indonesian, Chinese, German, English
- ✅ Audit log search and filter
- ✅ Data import: CSV → SQLite (migrate from other SCADA systems)
- ✅ Node-RED project importer: read `flows.json`, convert nodes to Voltrus tags and flow-engine rules
- ✅ Embedded MQTT broker (no external broker required)
- ✅ Light theme with industrial-friendly color scheme
- **Tier:** Free (tag browser, theme toggle) / Starter ($249, +import) / Professional ($499, +i18n)
- **Source (Node-RED):** r/SCADA — "look more to what Node-RED users are looking for" — they've outgrown it but can't afford Ignition

---

### v0.33.0 — Reliability & HA (COMPLETE)

**Goal:** "Take a shotgun to one server and nothing goes wrong."

- ✅ Formalized readback state machines for valve/pump control (prevents silent failures)
- ✅ Process watchdog/supervisor: auto-restart on crash, health check endpoints
- ✅ Hot standby: secondary Voltrus instance mirrors primary, auto-failover
- ✅ Config sync between primary and standby
- ✅ Failover indicator in dashboard (shows which node is active)
- ✅ Custom widget SDK (allow third-party dashboard widgets)
- ✅ Multi-network segment support: single Voltrus instance monitoring devices across segregated OT network zones
- **Tier:** Enterprise ($999)
- **Source (multi-network):** r/SCADA — "segregation means SCADA can't see assets lower in the network"

---

### v0.34.0 — OT Asset Inventory (COMPLETE)

**Goal:** Kill the spreadsheet. Lightweight asset tracking built into the SCADA system.

- ✅ Device inventory: auto-populated from configured devices (PLC, RTU, HMI, sensors)
- ✅ Firmware version tracking per device
- ✅ IP address inventory with conflict detection
- ✅ Maintenance history log: attach notes, dates, technician name to any device
- ✅ SNMP scanning for network-visible assets (switches, routers, gateways)
- ✅ NIS2 compliance export: device list with firmware, last-maintenance, risk score
- ✅ Asset tags: categorize by location, vendor, criticality, type (HVAC/power/controls)
- ✅ Firmware update flagging: alert when device firmware is outdated (compare against known versions)
- ✅ Excel/CSV import for existing asset spreadsheets (migration path from Excel-based inventory)
- ✅ Risk score engine with 4 weighted factors
- ✅ Device-to-asset auto-sync hooks for all protocols
- **Tier:** Professional ($499)
- **Source:** r/SCADA — "everyone uses Excel, nobody trusts it, NIS2 is forcing change"
- **Source (firmware):** r/SCADA — "most control systems are several versions behind on firmware"
- **Source (Excel import):** r/SCADA — "most sites start with Excel and run into the same problem"

---

### v0.35.0 — Documentation & System Mapping (BACKLOG — 4-5 weeks)

**Goal:** Reverse-engineer your own system. Auto-generated documentation for handover and compliance.

- Auto-generated system topology diagram (devices, connections, protocols)
- Tag cross-reference report: every tag with source device, data type, alarm config
- "What does this do?" — auto-document sequences and scripts with plain-English descriptions
- Export system documentation as PDF (for handover, audits, compliance)
- Config diff viewer: compare two config versions with highlighted changes
- **Tier:** Professional ($499)
- **Source:** r/PLC — "archaeological expedition through layers of undocumented AOIs" — engineers inherit undocumented systems and need to understand them fast

---

### v0.36.0 — Windows Defender OT Compatibility (COMPLETE)

**Goal:** Ensure Voltrus runs cleanly on Windows with Defender/EDR without false positives or performance issues.

- ✅ Document recommended Defender exclusions for Voltrus directories and processes
- ✅ Test with common OT EDR tools (Trellix ENS, FireEye, MS Defender for OT)
- ✅ Optimize file I/O patterns to minimize EDR scanning overhead
- ✅ Provide deployment guide for IT teams deploying to OT networks
- **Tier:** All tiers
- **Source:** r/SCADA — "xagt.exe (FireEye) deployed via AD to SCADA servers wasted CPU on monitoring communications"

---

### v0.37.0 — Vertical-Specific Templates (BACKLOG — 4-6 weeks)

**Goal:** Pre-built configurations for niche verticals to reduce time-to-value.

- Wastewater treatment template: pump stations, flow meters, chlorine dosing, lift stations
- Car wash template: conveyor control, chemical dosing, water recycling, payment integration
- Maritime template: bilge pumps, engine monitoring, navigation lights, tank levels
- Elevator/ski lift template: motor control, safety interlocks, position tracking, emergency stop
- Hydroelectric template: turbine control, gate position, flow monitoring, grid sync
- Each template includes: tag naming convention (ISA-5.1), default alarms, dashboard layout, OEE metrics
- **Tier:** Starter ($249, 1 template) / Professional ($499, all templates)
- **Source:** r/PLC — 98 comments on niche industries, most underserved by current SCADA vendors

---

### v0.38.0 — Hardware Certification & Edge Deployment Guide (COMPLETE)

**Goal:** "Runs on your existing hardware" — test, certify, document on real OT hardware.

- ✅ Test matrix: Raspberry Pi 4/5, Advantech UNO/IPC, Siemens SIMATIC IPC, generic fanless PCs
- ✅ Publish hardware compatibility matrix on docs site (RAM, CPU, storage per tier)
- ✅ Performance benchmarks per hardware tier (tag count, polling rate, compression throughput)
- ✅ Edge deployment guide: step-by-step for Raspberry Pi (flash SD, auto-start, headless mode)
- ✅ SD card write optimization: verify batch writes reduce write amplification under sustained load
- ✅ Partner outreach: SEL (RTAC), Advantech — explore pre-install/bundling opportunities
- **Tier:** All tiers
- **Source:** r/SCADA — Advantech most commonly used fanless HMI; r/PLC — "frequent writes without killing the SD card"

---

### v0.39.0 — External Integration & BI Connectors (COMPLETE)

**Goal:** Voltrus as the data source — connect to external BI, reporting, and analytics tools.

- ✅ Prometheus exporter endpoint (`GET /metrics`) with standard text exposition format
  - `voltrus_tag_value` gauge: current tag values with device, tag, and unit labels
  - `voltrus_device_up` gauge: device online/offline status (1/0)
  - `voltrus_alarms_active` gauge: active alarm counts by severity
  - No auth required (Prometheus scrapers typically can't authenticate), tier-gated internally
- ✅ REST API BI query endpoint (`GET /api/v1/integration/bi/query`)
  - Filtering by device IDs, metric names, time range (ISO 8601, unix timestamp, or relative like `now-1h`)
  - Pagination with limit/offset, JSON or CSV output format
  - Compatible with Power BI (Web connector), Metabase (REST API question)
- ✅ Grafana-compatible JSON endpoint (`GET /api/v1/integration/grafana/query`)
  - Returns `[value, timestamp_ms]` datapoint format for Grafana Infinity/JSON API data source
  - Metric search endpoint (`GET /api/v1/integration/grafana/search`) returns all device:metric targets
- ✅ Outbound webhooks for alarm and tag_change events
  - Configurable per-webhook: URL, event types (`alarm`, `tag_change`), device filter, custom headers, retry count, timeout
  - Automatic dispatch: alarm webhooks fire on every alarm, tag_change webhooks fire on each persistence tick
  - Exponential backoff retry, last-triggered status tracking
  - Test endpoint: `POST /api/v1/integration/webhooks/outbound/{id}/test`
- ✅ Configurable data egress: scheduled CSV/Parquet export
  - Export to filesystem (absolute/relative path) or S3-compatible storage (`s3://bucket/prefix/`)
  - Schedules: hourly, daily, weekly, or custom (`every_30m`, `every_2h`, `every_1d`)
  - Time ranges: 1h, 24h, 7d, 30d (configurable per job)
  - Optional gzip compression for CSV exports
  - S3 fallback: gracefully saves to local `./voltrus-egress` if S3 upload fails
  - Background scheduler runs every 60 seconds checking for due jobs
  - CLI: `voltrus egress list`, `voltrus egress run <ID>`
- ✅ Connection info endpoint (`GET /api/v1/integration/connection-info`)
  - Auto-generated Prometheus, Grafana, Power BI, Metabase connection details
  - Sample query URLs for quick testing
- ✅ Web UI: Integration panel in System view
  - Connection Info tab: Prometheus, REST API, Grafana, Power BI endpoints and sample queries
  - Webhooks tab: list outbound webhooks with test/delete + inline create form
  - Data Egress tab: list/manage export jobs with manual trigger + inline create form
- **Tier:** Starter ($249, outbound webhooks) / Professional ($499, Prometheus + BI connectors + data egress)
- **Source:** r/SCADA — "maintaining custom exporters for every OT protocol is becoming a bottleneck"; "Cannot justify the cost of heavy, traditional SCADA software/hardware licenses for every single pod"

---

### v0.40.0 — Project Wizards & FEED Templates (BACKLOG — 3-4 weeks)

**Goal:** Reduce time-to-value with guided project setup. No more blank-page syndrome.

- Project creation wizard: select industry → template → configure devices → generate dashboards
- FEED (Front-End Engineering Design) templates: pre-defined I/O lists, tag databases, alarm strategies
- Import wizard: step-by-step CSV/Excel import with column mapping and validation
- "Day 1" quickstart: connect first PLC, auto-discover tags, generate default dashboard in <10 minutes
- Project health check: audit existing config for common mistakes (missing alarms, unnamed tags, etc.)
- **Tier:** Starter ($249, basic wizard) / Professional ($499, FEED templates + health check)
- **Source:** r/PLC — "FEED study" mentioned as critical phase; r/SCADA — "start with blank system and bring up fast"

---

## Marketing Deliverables (Non-Code, Ship Alongside Roadmap)

These are content/marketing actions extracted from appendix research that should be produced alongside development:

| # | Deliverable | Source | Owner | Status |
|---|-------------|--------|-------|--------|
| 1 | **OT Security Checklist** (blog post — 15-step actionable checklist) | r/SCADA — Venice breach, "basic remediation checklist needed" | `blog/ot-security-checklist-scada.html` | **DONE** |
| 2 | **"Phase 1 Data Plumbing" blog post** | r/IndustrialAutomation — Voltrus is the data extraction layer for predictive maintenance | `blog/scada-data-extraction-predictive-maintenance.html` | **DONE** |
| 3 | **Perpetual License messaging** — stronger on landing page | Multiple threads — recurring fees are top complaint | `index.html` hero metrics + comparison table + subtitle | **DONE** |
| 4 | **Node-RED vs Voltrus comparison page** | r/SCADA — "look more to what Node-RED users are looking for" | `blog/voltrus-vs-node-red.html` | **DONE** |
| 5 | **"Mid-Market SCADA" blog post** — "The issue is not high, the issue is entry" | Walker Reynolds (4.0 Solutions) on AVEVA $50K/yr pricing + Alan White on mid-market being "vastly underserved" | `blog/scada-mid-market-entry-price.html` | **DONE** |
| 6 | **"Data Company First" blog post** — before AI, before analytics, get the data | Walker Reynolds core message + r/SCADA predictive maintenance thread | Covered by `blog/scada-data-extraction-predictive-maintenance.html` | **DONE** (merged into #2) |
| 7 | **"Cloud-Dependent Licensing is a Trap" blog post** — AVEVA forcing cloud migration, recurring costs | Walker Reynolds — "Aveva Changing Pricing?" video, every license becomes cloud-dependent | Covered by `blog/scada-mid-market-entry-price.html` (section: "Why Lifetime Licensing Matters") | **DONE** (merged into #5) |
| 8 | **Pricing page: "Entry price" messaging** — quote "the issue is not high, the issue is entry" | Walker Reynolds on AVEVA pricing | `index.html` pricing section | **TODO** |
| 9 | **Hardware compatibility matrix** | r/SCADA — fanless HMI thread, Advantech/SEL mentions | Docs page | **DONE** (v0.38) |
| 10 | **OT Network Deployment Guide** (security hardening, firewall rules, Defender exclusions) | r/SCADA — IT/OT bridging, Windows Defender thread | Docs page | **DONE** (v0.36) |

### YouTube Intelligence — Channels to Scrape

| Channel | Handle | Why | Priority |
|---------|--------|-----|----------|
| **4.0 Solutions** (Walker Reynolds) | `@4.0Solutions` | Industry 4.0 thought leader, ProveIt! conference host, shapes UNS conversation | **DONE** — 11 transcripts analyzed, Appendix G |
| **Automatic Revolution** | `@automaticrevolution` | SCADA-specific content, beginner-to-advanced playlists | **TODO** |
| **Inductive Automation** (Ignition) | `@inductiveautomation` | Direct competitor's official channel — product direction, marketing language | **TODO** |
| **Mike Holcomb** | `@UtilSec` | OT/ICS cybersecurity educator — insight into security buyer concerns | **TODO** |
| **RealPars** | `@realpars` | Largest automation education channel — what beginners ask about, content gaps to fill | **TODO** |
| **OT SCADA CON** | Conference channel | Direct community conference — speaker topics reveal market trends | **TODO** |

---

## Pricing Structure

### Core Licenses (Lifetime, No Recurring Fees)

| Tier | Price | Devices | Key Features |
|------|-------|---------|-------------|
| Free Trial | $0 | 5 | 30-day evaluation, all views, Voltrus branding |
| Starter | $249 | 50 | Modbus+MQTT, alarms, API, OEE, white-label |
| Professional | $499 | Unlimited | +OPC-UA, multi-site, SMS, PDF reports, RBAC |
| Enterprise | $999 | Unlimited | +SSO, S7, AB, CFR Part 11, RTSP video |

All tiers: lifetime license, no recurring fees, no per-tag charges.

### AI Copilot Add-on (Optional, Monthly)

| Tier | Price | Scope | Details |
|------|-------|-------|---------|
| BYOK | Free | Per machine | Bring your own OpenAI/Anthropic API key |
| Voltrus AI | $19/mo | Per engineer seat | Hosted inference, best models, unlimited deployments |

AI Copilot is optional. Core Voltrus works fully without it. See v0.26.5 for full details.

---

## Appendix A — Data Compression Strategy

**Problem:** At 10K tags @ 1Hz, raw SQLite = ~60GB/day. SCADA veteran claims "years of sub-second data, 30K+ points, in <200MB." Close this gap without abandoning SQLite.

**Decision:** Keep SQLite, add compression on top. No custom TSDB.

### Compression Tiers

| Tier | Data Age | Technique | Expected Ratio |
|------|----------|-----------|----------------|
| **Hot** | 0–24h | Raw + deadband filtering | ~5:1 |
| **Warm** | 1–30 days | Delta encoding + 1-min aggregates | ~50:1 |
| **Cold** | 30+ days | Gorilla delta-of-delta (numeric) + RLE (digital) | ~100:1 |
| **Frozen** | 1+ year | Archive to Parquet/CSV on disk | ~200:1 |

### Techniques

- **Deadband:** Per-tag configurable (0.1% analog, 0 digital). Skip INSERT if within deadband. Eliminates 60-90% of writes for steady-state data.
- **Delta encoding:** Store `(start_value, [(delta, ts_offset), ...])` per 1-min bucket. Varint deltas = 1-2 bytes vs 8-byte f64.
- **Gorilla (delta-of-delta):** If rate-of-change is constant, store a single bit. From Facebook's Gorilla paper (used by InfluxDB, Prometheus).
- **RLE for digitals:** `(value, count)` — pump running 8 hours = 1 row instead of 28,800.

### Storage Estimates (10K tags @ 1Hz)

| Tier | Compressed Size | Retention |
|------|----------------|-----------|
| Hot | ~12 GB/day | 1 day |
| Warm | ~240 MB/day | 30 days (~7 GB) |
| Cold | ~24 MB/day | 1 year (~8.6 GB) |

**Target: 1 year in ~15 GB total.**

### Implementation Steps

1. Add `deadband` field to tag config
2. Modify write path: skip INSERT when within deadband
3. Background compression worker (hourly): hot → warm
4. Cold compression worker (daily): warm → cold
5. Decompression layer in query path (transparent to frontend)
6. CLI: `voltrus db stats`, `voltrus db compress --now`, `voltrus archive`

### Why Not Replace SQLite?

- Handles 10K tags @ 1Hz easily with WAL mode
- Need relational queries for alarms, users, configs — TSDB can't do this
- Two databases = double maintenance and deployment complexity
- Compression wins come from encoding, not the storage engine
- 20 years of battle-testing matters for systems that "can never stop working"

---

## Appendix B — API & SDK Strategy

**Current:** REST+JSON with SSE for one-way real-time.

**Short-term (v0.24):** Command acknowledgement over REST+SSE.

| Capability | Implementation |
|-----------|----------------|
| Write with ack | `POST /api/v1/tags/{id}/write` → `202 Accepted` + `command_id` |
| Command tracking | `GET /api/v1/commands/{command_id}` → pending/sent/acked/failed |
| Real-time ack | SSE event `command_ack` to connected clients |
| Command history | `GET /api/v1/commands?tag=X&status=failed` |
| Offline queue | Queue writes, replay on reconnect |

**Medium-term (v0.29):** WebSocket for bidirectional streaming.

| Message | Direction | Payload |
|---------|-----------|---------|
| `tag_write` | Client → Server | `{tag_id, value, command_id}` |
| `tag_update` | Server → Client | `{tag_id, value, quality, ts}` |
| `command_ack` | Server → Client | `{command_id, status, plc_response}` |
| `device_status` | Server → Client | `{device_id, online, latency_ms}` |
| `alarm` | Server → Client | `{alarm_id, tag_id, severity, message}` |

**Why WebSocket, not gRPC?**
- gRPC-Web needs Envoy proxy — contradicts single-binary philosophy
- gRPC streaming overkill at SCADA rates (1-10Hz)
- WebSocket is browser-native, no extra infrastructure
- Can add gRPC alongside later if needed

**SDK Generation (v0.27):** OpenAPI-based.

1. Generate OpenAPI 3.1 spec from Rust handlers via `utoipa`/`okapi`
2. CI: on spec change → `openapi-generator-cli` → Python (PyPI) + TypeScript (npm)
3. Publish SDK docs and example scripts

| Feature | LiRAYS (Protobuf) | Voltrus (OpenAPI) |
|---------|-------------------|-------------------|
| Auto-generated types | Yes | Yes |
| Python / JS packages | PyPI + npm | PyPI + npm |
| Browser-native | Needs gRPC-Web proxy | Yes (axios/fetch) |
| Curl-friendly | No | Yes |

---

## Appendix C — Competitive Landscape & Reddit Insights

*Tracked: May 2026 — [github.com/LiRAYSgj/LiRAYS-Scada](https://github.com/LiRAYSgj/LiRAYS-Scada) (13 stars, Apache-2.0)*
*Source: [r/SCADA thread](https://reddit.com/r/SCADA/comments/1sz3s51/) — 18 upvotes, 16 comments*

### LiRAYS-SCADA

**Who:** Javier Castellon (u/HungryRegular6292) + Alejandro Alfonso, Costa Rica. Automation consultancy + open-source SCADA. Two-person team.

**Their stack:** Rust backend, `sled` embedded DB (not custom — they use the sled crate), Protobuf/gRPC, official SDKs (Rust/Python/JS), Docker-first, Astro docs.

**Their edge over us:** UNS architecture, protobuf APIs, multi-language SDKs, fully open-source (Apache-2.0).

**Our edge over them:** 5 protocols (they have Modbus + MQTT + partial OPC-UA), full auth/RBAC/SSO, OEE, PDF reports, multi-site, CFR Part 11, IEC 62443, PWA, licensing/monetization.

**Threat level:** LOW — 13 stars, 2 devs, no commercial traction.

### Reddit Thread — Full Insight Extraction

**Top comment — EmperorOfCanada (score:8, SCADA software veteran):**

Key insights already in roadmap (simulation mode, compression, scripting, rollback). Additional insights we should track:

| Insight | Implication for Voltrus |
|---------|----------------------|
| "Look more to what Node-RED users are looking for" | **Marketing:** Study Node-RED community pain points — they're our exact target market (can't afford SCADA, want something better than Node-RED) |
| "MQTT was built for SCADA... MQTT is very uncommon in the industrial world" | **Product:** We have MQTT, but shouldn't over-index on it. Modbus + OPC-UA are what industrial users actually deploy. MQTT is our IoT/edge play. |
| "SCADA operators want their own user admin and won't let IT anywhere near their SCADA system" | **Product:** Our RBAC is correct. SCADA IT ≠ corporate IT. Our local auth (Argon2) + separate RBAC is the right architecture. Don't over-integrate with corporate AD. |
| "Make your own protocol drivers, one at a time" | **Product:** Validates our approach — native Rust protocol drivers, not wrappers around Kepware or Matrikon |
| "The moment you tightly tie into some crap like kepware, you will regret ever being born" | **Product:** Never build on top of middleware. Own the protocol stack. |
| "A system to do a customer install... need to start with a blank system and be able to bring up a blank server as fast as possible" | **Product:** Our backup/restore CLI is good, but needs to be faster. Consider `voltrus factory-reset` + `voltrus restore --from-backup` one-liner for disaster recovery. |

**PeterHumaj (score:2, Ada developer, 23 years building SCADA):**
- Uses Ada (not Rust) for a SCADA system originally from 1993, rewritten 1998, still running
- "Readability, strong type control, maintainability... customers use for decades"
- **Implication:** Longevity matters in SCADA. Our code needs to be maintainable for 10+ year deployments. Rust's type system is an advantage here.
- **Implication:** Competition isn't just new tools — it's 30-year-old systems still in production. We need migration/import tools.

**Nearby_Raspberry_831 (score:2, edge/industrial IoT):**
- "Backend logic usually falls apart at 1,000+ tags"
- "Frequent writes without killing the SD card" — flash storage optimization for Raspberry Pi deployments
- **Implication for v0.25 (compression):** Add write-amplification awareness for flash/SD card storage. Batch writes, avoid tiny random writes that kill SD cards.

**LiRAYS founder responses (u/HungryRegular6292):**
- Uses **sled** (not custom DB) — an embedded key-value store in Rust, similar to RocksDB
- "We didn't build amazing algorithms... For now we rely on Rust strength, focusing on performance in first place (keeping code readability in second place)"
- "We will explore possible optimizations for flash-based storage"
- **Implication:** Their DB story is weaker than we assumed. Sled is a KV store, not a time-series DB. SQLite with our compression strategy is genuinely better.

**Competitor mentions in thread:**
- **FUUXA** (asked "how is it better than Fuxa?") — open-source web SCADA, our direct competitor at the open-source level
- **Inductive Automation / Ignition** (score:6) — the elephant in the room. "cough cough" = market leader
- **N3uron** — commercial, recurring costs
- **Open Automation System (OAS)** — claims 100K tags on Raspberry Pi 4, perpetual license
- **vNode Automation** — perpetual licenses, optional tech support

**Key market signal:** Comment about Ignition's "recurring costs per year" (score:1) and multiple mentions of perpetual licensing. **Our lifetime license model is a genuine differentiator** — make this louder in marketing.

### Voltrus Positioning (Updated)

1. **"Affordable but serious"** — not $50K Ignition, not a toy like FUUXA
2. **Lifetime license** — no recurring fees (competitors charge yearly)
3. **Single binary, bare-metal** — veteran explicitly said "go local, cloud backup for emergencies"
4. **Own the protocol stack** — no Kepware/Matrikon dependency
5. **Target Node-RED users** — they've outgrown Node-RED but can't afford Ignition
6. **Edge-friendly** — Raspberry Pi + flash storage optimization matters

---

## Appendix D — Industry Insights from Reddit (May 2026)

*Sourced from r/SCADA, r/PLC, r/industrialautomation — 8 threads, ~140 comments total.*

### Thread 1: "Things I wish someone told me before I designed my first SCADA system" (score:46, 29 comments)

**Key insights:**

| Insight | Implication for Voltrus |
|---------|----------------------|
| "Tag naming will haunt you" — ended up with 3 different names for same tag, caught at 8K tags deep. ISA-5.1 standard. | **v0.32 tag browser** should enforce naming conventions. Offer ISA-5.1 template out of the box. |
| "Design for the plant you'll have, not the one you have now" — pilot had 50 I/O, production 4,000. Historian choked. | Our compression + performance work (v0.25) handles this. Market as "scales from pilot to production." |
| "If the operators don't trust it, they'll route around it" — operators bypassed the system within a month. | HMI must be dead simple. Our touch-first 48px targets are right. |
| "Design for the dumbest guy in the room" — all the complex stuff in sub-menus | Reinforces our simple dashboard approach. Noted for light theme (v0.32). |
| "Alarm fatigue is the real killer" — operators disabled alarms, then missed critical ones for months | Our alarm system needs: non-repeating alarms, alarm shelving with auto-restore, alarm summary dashboard. Consider for v0.24 or backlog. |
| Red/green fails for colorblind users — "shape plus color is the fix" | **v0.32 light theme** should use shape+color indicators, not color alone. |
| "A screen that needs a manual is a screen that needs a redesign" | Product must be self-explanatory. No training needed (we cut certification, this validates it). |
| ISA-101 (HMI best practices) and ISA-62443 (cyber security) should be read before any project | We already ship IEC 62443 (v0.22). ISA-101 is a design guide for our HMI. |

### Thread 2: "OT Asset Documentation — what's your current workflow?" (score:11, 24 comments)

**Key insight:** Everyone uses Excel. Nobody trusts it. NIS2 compliance is forcing change. Tools mentioned: Claroty, Nozomi, Fortinet, phpIPAM, NetBox, Lansweeper OT.

**Implication for Voltrus:** Device inventory management is an unsolved pain point. Our device management (Modbus auto-discovery, device configs) could evolve into a lightweight OT asset tracker:
- Firmware version tracking
- IP address inventory
- Maintenance history log
- NIS2 compliance export
- **Consider for backlog** — not core SCADA but a natural extension of device management.

### Thread 3: "Ignition Historian with TimescaleDB" (score:4, 11 comments)

**Key insight:** Someone running 20 MILLION tags with Ignition + TimescaleDB. TimescaleDB gives ~90% compression. External systems can query the single table directly (vs. Ignition's partitioned format).

**Implication:** Our compression strategy (Appendix A) is the right call. The market is moving toward SQL-compatible historians. SQLite + compression = simpler, no external DB needed.

### Thread 4: "OT Cybersecurity" (score:13, 34 comments)

**Key insights:**

| Insight | Implication |
|---------|-------------|
| "Keep IT out of your network if you can" — IT updates break OT systems | Our RBAC + separate SCADA admin is correct. Don't over-integrate with corporate IT. |
| "Zero trust at the firewall for most OT systems" | We're well-positioned. Our IEC 62443 (v0.22) + RBAC + API key auth covers this. |
| Passive monitoring > active scanning — "active scanning has knocked things offline before" | Our device monitoring should be passive (poll/read-only) by default. Write commands should be explicit opt-in. |
| IEC 62443 / SL3/SL4 compliance requested for wind farms | Our v0.22 security audit mode is marketable. Emphasize in energy vertical. |
| Fortinet can "block Modbus config commands while letting data through" | Interesting — protocol-level firewall. Not our scope but partner opportunity. |

### Thread 5: "Predictive Maintenance — vendors won't tell you what it actually takes" (score:13, 15 comments)

**Key insights:**
- 18-30 month realistic timeline for meaningful predictive maintenance
- Phase 1 (data collection from legacy PLCs) kills most projects
- "Keep PLC for control, use independent gateways for data-heavy lifting"
- Sensor companies moving toward signal capture at sensor layer (SICK Sentio)
- "70% of firmware updates aimed at trade show simulations while shop floor stays broken"

**Implication:** Voltrus could own the "Phase 1" problem — get data OUT of PLCs cheaply and reliably. Our multi-protocol support (Modbus, OPC-UA, S7, AB) + affordable pricing + edge-friendly = we're the data plumbing that makes predictive maintenance possible. Marketing angle: "Your predictive maintenance journey starts with getting the data. That's us."

### Thread 6: "Bridging IT and OT at scale — small decentralized data centers" (score:7, 19 comments)

**Key insights:**
- Company with small "pods" (PLC + networking + server racks), 50-100 tags each, scaling to many sites
- Current stack: Prometheus + custom exporters — "maintaining custom exporters for every OT protocol is becoming a bottleneck"
- "Cannot justify the cost of heavy, traditional SCADA software/hardware licenses for every single pod"
- "PLC protects the process; edge validates and contextualizes the data"

**Implication:** This is our exact customer. Multi-site, small per-site tag count, can't afford Ignition. Our Professional tier ($499, multi-site, unlimited devices) is perfect for this. The "custom protocol exporters" pain is exactly what Voltrus eliminates with native protocol drivers.

### Thread 7: "3D Interactive Dashboards" (score:9, 5 comments)

**Key insight:** Skeptical reception. "Why?", "A screen to re-render other screens", "Super cool but can't come up with much of a use case."

**Implication:** Our 3D Digital Twin view is a differentiator but don't over-index on it. The market wants simple 2D HMI (red/green, status overview). 3D is a "wow" demo feature, not a purchase driver. Our grid/canvas views are more valuable than 3D.

### Thread 8: "Modbus stack in Rust" (score:3, 6 comments)

**Key insight:** Another Rust developer building a full Modbus stack (TCP/RTU/ASCII, async+sync, embedded-first, no_std). Includes FIFO, file records, diagnostics. Has a desktop client for testing.

**Implication:** The Rust industrial ecosystem is growing. We should track `modbus-rs` crate as a potential replacement/enhancement for our Modbus implementation if it matures. Also validates our Rust choice — the community is building the building blocks.

---

## Appendix E — Marketing Signals Summary

**What SCADA users complain about:**
1. Tag naming chaos — no enforced standards
2. Alarm fatigue — too many false alarms, operators disable them
3. Cost — Ignition is expensive, recurring fees suck
4. Legacy PLC data extraction — getting data out is the bottleneck
5. IT/OT conflict — IT breaks OT systems with updates
6. Operator distrust — if HMI is confusing, operators bypass it
7. Asset documentation — everyone uses Excel, nobody trusts it
8. Scaling — pilot to production breaks architectures

**What SCADA users want (that Voltrus can offer):**
1. Affordable, perpetual license (no recurring fees)
2. Simple HMI that operators trust (not engineer playgrounds)
3. Native protocol drivers (no Kepware/Middleware dependency)
4. Edge-friendly (runs on Raspberry Pi / fanless PCs)
5. Multi-site without per-site licensing
6. Data compression for long-term history
7. Security that doesn't break operations (passive, not active scanning)

**Competitors mentioned across all threads:**
- Ignition / Inductive Automation (market leader, expensive)
- FUUXA (open-source web SCADA)
- OAS / Open Automation System (100K tags on RPi4)
- WinCC Unified (Siemens)
- N3uron (recurring costs)
- vNode Automation (perpetual, optional support)
- Rapid SCADA (open-source)
- Node-RED (not SCADA but used as one)

---

## Appendix F — Fresh Reddit Insights (May 2026 — Continuous Scrape)

*Source: r/SCADA, r/PLC, r/IndustrialAutomation, r/ControlEngineering — 12 threads, ~350 comments total.*

### Thread 1: "Niche/little known industries in automation" (r/PLC, score:26, 98 comments)

**Key insight:** Massive untapped verticals beyond the usual Oil & Gas / Food & Bev / Automotive / Pharma.

| Niche Vertical | Voltrus Opportunity |
|---|---|
| Wastewater & drinking water (municipal) | Natural fit — SCADA-heavy, budget-conscious, needs multi-site |
| Car washes (tunnel + in-place) | Small footprint, needs reliability, can't afford Ignition |
| Elevators / people-moving (ski lifts, roller coasters) | Safety-critical, needs audit trail + alarm management |
| Maritime / shipboard controls | Edge-friendly, offline-capable, single-binary |
| Amusement parks | High-reliability, multi-site, seasonal scaling |
| Hydroelectric generators | IEC 62443 compliance, DNP3 protocol (v0.30) |
| Welding automation | Small-to-mid shops, can't afford enterprise SCADA |
| Printing / web tension | Specialized but underserved |
| Lab-grown diamonds | Emerging industry, greenfield deployments |
| Munitions / defense | Security-compliance heavy, CFR Part 11 relevant |
| Agricultural / fertilizer manufacturing | Rural deployments, edge-friendly matters |

**Implication:** Our marketing should target these verticals specifically. "Affordable but serious" resonates strongest in niche markets where Ignition pricing is unjustifiable.

### Thread 2: "The most convoluted system I've ever worked with" (r/PLC, score:55, 36 comments)

**Key insights:**

| Insight | Implication for Voltrus |
|---|---|
| "Archaeological expedition through layers of undocumented AOIs" | **v0.35 Documentation** — auto-generate system maps, tag cross-references |
| "Every output, input, analog described as AI.#, AO.# — no descriptions" | Tag browser should require descriptions on creation. Enforce naming conventions. |
| "Sometimes it's best to start from scratch... you'll wish you did" | Simulation mode (v0.26) lets engineers test before deploying. Add config diff viewer. |
| "Two words: FEED study" | Front-End Engineering Design — we should provide project templates / wizards |
| "Completely ignoring the HMI at first. Map physical I/O to base routines" | Tag browser with device-centric view (not just flat list) helps understand the system |
| OEMs walking away from incomplete projects, leaving customers stranded | **Marketing angle:** "No vendor lock-in. Own your system. We won't disappear." |

### Thread 3: "Vendors will tell you their platform does predictive maintenance" (r/IndustrialAutomation, score:13, 15 comments)

**Key insights (reinforces existing roadmap positioning):**

| Insight | Implication |
|---|---|
| "6-12 months extracting data from legacy PLCs" | **Voltrus IS the Phase 1 solution.** Multi-protocol support + affordable = data plumbing |
| "Keep PLC for control, use independent gateways for data-heavy lifting" | Voltrus architecture validates this — we're the independent gateway |
| "Raw sensor readings are noisy and inconsistent" | Deadband filtering (v0.25) directly addresses this |
| "Baseline takes time — that's where people draw wrong conclusions" | Our OEE + downtime tracking provides the baseline that predictive maintenance needs |
| "Dashboards break but production rolls on, we'll fix dashboards later" | Reliability first. Our single-binary, no-external-dependencies approach is correct |

**Marketing angle:** "Your predictive maintenance journey starts with getting the data. That's us."

### Thread 4: "Software-Defined Automation — real or buzzword?" (r/IndustrialAutomation, score:7, 8 comments)

**Key insights:**

| Insight | Implication |
|---|---|
| "Never used it, never seen it in a factory" | Not a priority. The market isn't ready. Stick with proven PLC paradigms. |
| "Determinism, latency, debugging across layers became real concerns" | Don't abstract away from the control layer. Our direct protocol drivers are correct. |
| "Most teams are still more comfortable with traditional PLC workflows" | Sandboxed scripting (v0.26) should be optional, not the primary paradigm. |
| "PC-based automation has been 'about to take over' for 30 years, still niche" | Single binary on bare metal is our positioning. Don't chase virtualization. |
| "Big companies will want it for failsafe migration and documentation" | Documentation features (v0.35 backlog) address this need without changing architecture. |

### Thread 5: "3D Interactive Dashboards" (r/IndustrialAutomation, score:8, 5 comments)

**Key insights (CONFIRMS earlier finding):**

| Insight | Implication |
|---|---|
| "Why?" | Don't lead with 3D. It's a demo feature, not a purchase driver. |
| "A screen to re-render other screens" | 3D adds abstraction layer between operator and reality. |
| "Can't come up with much of a use case" | Our 3D Digital Twin view is fine to keep, but don't market it heavily. |
| "Super cool but trouble than it's worth" | **Reinforces:** Grid view + Canvas view are the primary views. 3D is secondary. |

### Thread 6: "OT Asset Documentation — what's your workflow?" (r/SCADA, score:10, 24 comments)

**Additional insights beyond what's already in v0.34:**

| Insight | Implication |
|---|---|
| "We integrated SNMP into our SCADA templates" | SNMP scanning (already in v0.34) is the right approach. |
| "Can't get back everything we'd like, some don't report back" | Asset inventory should handle "unknown/unreachable" gracefully. |
| "Segregation means SCADA can't see assets lower in the network" | Multi-network support — Voltrus may need to run on multiple network segments. |
| "Most sites start with Excel and run into the same problem" | Import from Excel as a migration path into our asset inventory. |
| "Use SharePoint as structured asset register... Power BI on top for visibility" | Our CSV export for NIS2 is correct. Consider API for external BI tools. |

**Competitor tools mentioned:** Claroty xDome, Nozomi, Fortinet OT, phpIPAM, Lansweeper OT, SEL Blueframe DMA, Automation Asset Manager.

### Thread 7: "OT Cybersecurity" (r/SCADA, score:13, 34 comments)

**Additional insights beyond v0.22:**

| Insight | Implication |
|---|---|
| "Most control systems are several versions behind on firmware" | Asset inventory should flag outdated firmware as a security risk. |
| "Cybersecurity lives at the firewall for most OT systems" | Our IEC 62443 audit mode is correct for what SCADA controls. |
| "Passive network monitoring is preferred over active scanning" | Our device monitoring should be poll/read-only by default. Already our architecture. |
| "Fortinet can block Modbus config commands while letting data through" | Partner opportunity — Fortinet OT add-on for protocol-level filtering. |
| "Microsoft Defender for OT — may be eligible for free licenses" | Our Windows deployment should be Defender-compatible (add exclusion recommendations). |
| "Waterfall's unidirectional gateway — data out, nothing in" | Air-gap friendly (already in our licensing) aligns with this architecture. |

### Thread 8: "IT/OT Bridging at Scale — small decentralized data centers" (r/SCADA, score:8, 19 comments)

**Key insights:**

| Insight | Implication |
|---|---|
| "Edge compute + Prometheus + exporters is a solid foundation" | Our single binary is effectively the "edge exporter" for all protocols. |
| "Maintaining custom exporters for every OT protocol is becoming a bottleneck" | **This is our value prop.** One binary, all protocols. No custom exporters needed. |
| "Cannot justify heavy SCADA licenses for every single pod" | Professional tier ($499, multi-site, unlimited) is exactly this use case. |
| "Data validation should happen at PLC level if possible" | Our Data Flow Engine validation rules align with this. |
| "Challenge the idea of centralizing HMIs — think about where logic lives" | Multi-site hub dashboard (already shipped) + local autonomy is correct. |

### Thread 9: "Ignition Historian with TimescaleDB" (r/IndustrialAutomation, score:13, 4 comments)

**Key insights:**

| Insight | Implication |
|---|---|
| "TimescaleDB has advantages over plain SQL for time-series" | Validates our compression strategy (Appendix A) — encoding matters more than engine. |
| "Smaller footprint than using SQL database directly" | SQLite is even smaller than Postgres+Timescale. Our approach is leaner. |
| "Cheaper than Aveva PI" | **Our compression target:** 1 year of 10K-tag data in ~15GB, no external DB needed. |

### Thread 10: "AI/LLM connected to plant data" (r/PLC, score:1, 6 comments)

**Key insight:** The top comment (score:10) was "Fuxk no!" — overwhelming resistance to AI in OT environments.

**Implication:** **Do NOT build AI/LLM features into Voltrus.** The community sees AI as a risk to safety-critical systems. Our deterministic, transparent approach is the correct positioning.

### Thread 11: "Fanless HMI Computers" (r/SCADA, score:6, 14 comments)

**Key insights:**

| Hardware Vendor | Notes | Voltrus Implication |
|---|---|---|
| SEL RTAC | Rugged, 10yr warranty, pricey | Partner opportunity — pre-install Voltrus on SEL hardware |
| Advantech | Most commonly used, good warranty | Test Voltrus on Advantech hardware for edge deployments |
| Beckhoff | Used for years, good support | Already support via OPC-UA |
| Siemens | Good bulk discounts | Already support via S7 protocol |

**Implication:** Test and certify Voltrus on Advantech edge hardware. Market as "runs on your existing hardware — no special requirements."

### Thread 12: "Venice Flood Control OT Breach" (r/IndustrialAutomation, score:16, 3 comments)

**Key insights:**

| Issue | Implication |
|---|---|
| "Exposed HMIs, default creds, no segmentation, zero monitoring" | Our IEC 62443 audit mode directly addresses: default creds (no defaults), monitoring (security events) |
| "Basic remediation checklist needed" | Consider providing an OT security checklist as a free marketing resource |
| "Administrative access left exposed for months" | Our security event logging (v0.22) with configurable alert thresholds catches this |

---

## Updated Voltrus Positioning (May 2026)

1. **"Affordable but serious"** — not $50K Ignition, not a toy like FUUXA
2. **Lifetime license** — no recurring fees (competitors charge yearly)
3. **Single binary, bare-metal** — go local, cloud backup for emergencies
4. **Own the protocol stack** — no Kepware/Matrikon dependency
5. **Target Node-RED users** — they've outgrown Node-RED but can't afford Ignition
6. **Edge-friendly** — Raspberry Pi + flash storage optimization matters
7. **"Phase 1" data plumbing** — get data OUT of PLCs cheaply and reliably
8. **No AI gimmicks** — deterministic, transparent, safety-first
9. **Target niche verticals** — wastewater, car washes, maritime, elevators, hydroelectric
10. **Documentation is a feature** — engineers inherit undocumented systems daily

---

## Landing Page Files to Update Per Release

After each release:
- `voltrus-landing/changelog.html` — add version entry
- `voltrus-landing/docs.html` — update docs for new features
- `voltrus-landing/index.html` — update `softwareVersion` in schema.org, feature list if changed

---

## Appendix G — YouTube Competitive Intelligence (May 2026)

*Source: [4.0 Solutions](https://www.youtube.com/@4.0Solutions) (Walker Reynolds) — 30 videos scraped, 11 transcripts analyzed. Walker Reynolds is a leading Industry 4.0 consultant with significant influence in the SCADA/OT community.*

### Channel Profile

**Who:** Walker Reynolds, Industry 4.0 consultant. Runs 4.0 Solutions — consulting, training, and the annual ProveIt! conference. Not a SCADA vendor, but a thought leader shaping the Industry 4.0 conversation.

**Audience:** Manufacturing engineers, system integrators, plant managers — our exact target market.

**Key themes:** UNS (Unified Namespace), data-first transformation, anti-hype positioning on AI, AVEVA/Ignition competitive dynamics.

### Key Insights for Voltrus

| Video | Insight | Implication for Voltrus |
|-------|---------|----------------------|
| **"AI isn't helping..."** (3w ago) | "Fuck AI... I don't say that because AI is bad... it's because of what some people and organizations are doing with it." Market problems haven't changed in 5-8 years: becoming data companies to automate business decisions. | **Validates our no-AI stance.** The market wants data plumbing, not AI features. Our positioning as "Phase 1 data extraction" is exactly right. |
| **"This Breaks the Entire Industrial Software Model"** (1mo ago) | CESMII (Smart Manufacturing Institute) presenting on standards becoming critical for UNS. John Dyck (CEO) accidentally shut down a Kellogg's plant — shows real-world stakes. | **Standards matter.** CESMII's work on smart manufacturing profiles could influence what data models Voltrus should support. Track CESMII standards for future tag/data models. |
| **"The Industry 4.0 Playbook is Wrong!"** (1mo ago) | Alan White (VP IT, LSB Industries): "The sweet spot I love operating in is really in this mid-market space. I think it's just such a vastly underserved market." Playbooks that work are reverse-engineered from failures. | **Mid-market is our exact customer.** Validate our pricing ($249-$999) vs enterprise ($6K+). Our positioning as "affordable but serious" matches the underserved mid-market. |
| **"Aveva Changing Pricing?"** (2mo ago) | AVEVA Ops Control: ~$50K/year for site edition (25 users). Walker says "that's a completely fair number... the issue is not high. The issue is entry." Every license becomes cloud-dependent, forcing recurring costs. | **Entry price is the battleground.** AVEVA's $50K/year makes our $249 lifetime look absurdly cheap — in a good way. Emphasize "entry" in marketing: no barrier to start. |
| **"STOP..."** (3mo ago) | Top 3 mistakes manufacturers make in 2026: (1) starting with technology instead of business problems, (2) treating Industry 4.0 like IT projects, (3) skipping foundational data architecture. | **Voltrus IS foundational data architecture.** We're not the AI layer, we're the data foundation. Marketing: "Before you can transform anything, you need the data." |
| **"Why So Hard?"** (3mo ago) | Successful pilots fail to scale because: (1) 3IR mindset applied to 4IR problems, (2) no hardened processes for data decisions unlike industrial processes, (3) missing foundational data architecture. | **Scales from pilot to production.** Our pricing doesn't penalize scaling — $499 Professional is unlimited devices. This is a genuine differentiator vs per-tag pricing. |
| **"UNS is the GOAT!"** (1mo ago) | "Why doesn't shop floor data translate into better business decisions? It's either not real time, it's not in context, or it's not related to the decision you need to make." | **Real-time + context is our value.** Voltrus provides real-time data (1Hz polling, SSE streaming). Context (tag naming, asset grouping) is coming in v0.32/v0.34. |
| **"Autonomous Agents..."** (4mo ago) | "Are autonomous agents the future of industry? The answer is fuck no." Best LLMs are 99.9% accurate = 1 error per 1000 words. In industry, 1 error can kill people. Semi-autonomous agents (human-in-the-loop) are the future. | **Deterministic + transparent.** Voltrus never makes autonomous decisions. Our alarm routing, notification logic, and data flow engine are all human-configured, deterministic rules. This is the correct approach. |
| **"2025 Said What..."** (3mo ago) | "If you think MCP is not important, you are a fool. You must know Model Context Protocol. You will end your career if you do not know it." | **Track MCP but don't implement.** Model Context Protocol is for AI agent contexts. Our no-AI stance means we don't need it, but our API should be structured so external MCP clients could query Voltrus data. |
| **ProveIt! 2026 Conference** (4-day recap) | Major annual Industry 4.0 conference. AVEVA, Litmus, HiveMQ, Flow Software, CESMII all present. Walker's community is the epicenter of the UNS/data transformation movement. | **Conference opportunity.** ProveIt! is where our potential customers gather. Consider sponsoring or presenting in future years when Voltrus has more market traction. |

### Competitive Signals

| Signal | Source | Implication |
|--------|--------|-------------|
| AVEVA moving to cloud-dependent licensing | "Aveva Changing Pricing?" video | **Opportunity.** Every AVEVA customer being forced into recurring cloud licenses is a potential Voltrus customer who wants to own their stack. |
| Walker Reynolds is anti-AI-hype but pro-data-foundation | Multiple videos | **Alignment.** Walker's message is "become a data company first" — Voltrus is the tool to do that. Potential partnership/endorsement opportunity. |
| "Mid-market is vastly underserved" | Alan White, LSB Industries | **Confirmation.** Our target market is validated by multiple independent sources (Reddit, YouTube, SCADA veterans). |
| UNS architecture becoming mainstream | Multiple videos + CESMII | **Track for v0.32.** Our tag browser should support UNS-style hierarchical naming. Already planned. |
| Industry 4.0 community standardizing around MQTT + UNS | Multiple videos | **We have MQTT.** UNS-style tag browser in v0.32 aligns with this trend. No need to build a full UNS platform — just structured tag naming. |

### Actionable Items for Voltrus

1. **Marketing:** Quote Walker's "the issue is not high, the issue is entry" in our pricing section — our $249 entry point solves this exact problem
2. **Marketing:** "Become a data company first" is Walker's core message. Our "Phase 1 Data Plumbing" blog post aligns perfectly
3. **Product (v0.32):** UNS-style tag naming in the tag browser — already planned, validated by Walker's community
4. **Product (v0.38):** API structured for external tool consumption (MCP clients, BI tools) — track but don't implement AI features
5. **Business:** Monitor ProveIt! conference for potential sponsorship/presentation opportunity as Voltrus gains traction
6. **Content:** Engage with Walker's community on LinkedIn/YouTube — our "affordable but serious" positioning matches his data-first message
