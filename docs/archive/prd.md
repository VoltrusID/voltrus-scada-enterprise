> Archived 2026-10-10: original v1 PRD (Modbus TCP -> HTTP, 2 devices) superseded by the multi-protocol platform described in README.md.

Good call going with HTTP first—this keeps your system simple, observable, and cheap. I’ll give you a concise but production-minded PRD you can actually build from.

⸻

📄 PRD — Rust Modbus → HTTP Service (SCADA-lite / PLTS)

1. 🎯 Objective

Build a lightweight Rust service that:

* polls Modbus TCP devices
* normalizes data into semantic metrics
* exposes data via HTTP API for Cloudflare Pages UI
* optionally stores historical data (libSQL)

⸻

2. 🧩 Scope (v1)

Included

* Modbus TCP polling (multi-device)
* Config-driven register mapping
* In-memory cache (latest values)
* HTTP API:
    * live data
    * basic history
* Optional persistence (libSQL)

Not included (v1)

* WebSocket
* MQTT
* advanced alerting
* multi-tenant auth (handled by Cloudflare)

⸻

3. 🏗️ System Architecture

[Smart Logger]
    ↓ Modbus TCP
[Rust Service]
    ├─ Poller (tokio-modbus)
    ├─ Cache (in-memory)
    ├─ Storage (libSQL)
    └─ API (axum)
         ↓ HTTP
[Cloudflare Pages UI]

⸻

4. ⚙️ Functional Requirements

4.1 Device Configuration

Use a config file (YAML or JSON).

Example (YAML)

devices:
  - id: "plts_1"
    ip: "192.168.1.10"
    port: 502
    unit_id: 1
    interval_ms: 1000
    registers:
      - name: "voltage"
        address: 40001
        type: "float32"
        scale: 0.1
      - name: "current"
        address: 40003
        type: "float32"
        scale: 0.01
      - name: "power"
        address: 40005
        type: "float32"

⸻

4.2 Polling Engine

* async (Tokio)
* per-device loop
* interval configurable (default: 1s)

Behavior:

* connect → read registers → parse → normalize
* update cache
* optionally persist

Error handling:

* retry on failure
* exponential backoff
* mark device as “offline” if timeout

⸻

4.3 Data Normalization

Convert raw Modbus registers into:

{
  "device_id": "plts_1",
  "timestamp": 1710000000,
  "metrics": {
    "voltage": 220.5,
    "current": 5.2,
    "power": 1146
  }
}

⸻

4.4 In-Memory Cache

* structure:

HashMap<DeviceId, LatestData>

* always serve /api/live from cache
* avoid DB read for real-time

⸻

4.5 Storage (libSQL / SQLite)

Table: telemetry

CREATE TABLE telemetry (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  device_id TEXT,
  timestamp INTEGER,
  metric TEXT,
  value REAL
);
CREATE INDEX idx_device_time ON telemetry(device_id, timestamp);

Write strategy:

* batch insert every N seconds (e.g. 1–5s)
* async queue

⸻

5. 🌐 API Specification

5.1 GET /api/live

Returns latest snapshot

{
  "devices": [
    {
      "device_id": "plts_1",
      "status": "online",
      "timestamp": 1710000000,
      "metrics": {
        "voltage": 220.5,
        "current": 5.2
      }
    }
  ]
}

⸻

5.2 GET /api/history

Query:

/api/history?device_id=plts_1&metric=voltage&from=...&to=...

Response:

[
  { "timestamp": 1710000000, "value": 220.1 },
  { "timestamp": 1710000060, "value": 221.0 }
]

⸻

5.3 GET /api/health

{
  "status": "ok",
  "devices": {
    "plts_1": "online"
  }
}

⸻

6. 🔌 Modbus Handling Details

Register Types

Support:

* int16
* uint16
* int32
* float32 (most important for PLTS)

⸻

Endianness (IMPORTANT)

Allow config:

endianness: "big" # or little / swapped

⸻

Function Codes

* Read Holding Registers (0x03)
* (optional later) Input Registers (0x04)

⸻

⚙️ 7. Non-Functional Requirements

Performance

* support ≥ 50 reads/sec
* API latency < 50ms (cache-based)

Reliability

* reconnect on failure
* device status tracking

Resource target (your VPS)

* RAM: < 300MB
* CPU: low

⸻

🔐 8. Security

* run behind Cloudflare Tunnel
* no public port exposed
* optional:
    * API key header for internal protection

⸻

🧱 9. Tech Stack

* Runtime: Tokio
* Modbus: tokio-modbus
* API: axum
* DB: libsql or rusqlite
* Serialization: serde

⸻

🚀 10. MVP Milestones

Phase 1 (1–2 days)

* config loader
* single device polling
* /api/live

Phase 2

* multi-device
* cache + status
* basic error handling

Phase 3

* libSQL persistence
* /api/history

⸻

💡 Design Principles (important)

* decouple protocol from API
* cache first, DB second
* config-driven (no hardcoding registers)
* keep ingestion stateless where possible

⸻

🧠 Final sanity check (your scale)

You said:

* 2 devices
* 10 datapoints each

👉 This system will be:

* extremely fast
* extremely cheap
* very stable

⸻

If you want, next step I can:

* ￼ give you actual Rust project structure (folder + modules)
* or ￼ write starter code (poller + axum API skeleton)

That’ll get you from PRD → running service quickly.
