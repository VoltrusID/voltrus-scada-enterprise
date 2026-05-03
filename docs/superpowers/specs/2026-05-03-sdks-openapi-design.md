# v0.27.0 — SDKs & OpenAPI

**Date:** 2026-05-03
**Status:** Approved
**Timeline:** 3-4 weeks
**Tier:** Free (SDKs are open-source, API requires any paid tier)
**Depends on:** v0.26.0 (Sandboxed Scripting & Simulation) for merge coordination

---

## Goal

Match LiRAYS developer experience. Official SDKs, auto-generated from API spec via OpenAPI 3.1. Engineers integrate Voltrus data into Python scripts, TypeScript apps, and external tools with minimal effort.

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| OpenAPI generation | utoipa (procedural macros) | Most popular Rust OpenAPI lib, Axum support, OpenAPI 3.1, opt-in per handler |
| SDK languages | Python (PyPI) + TypeScript (npm) | The two most common integration languages for SCADA/OT |
| API explorer | Embedded Swagger UI (utoipa-swagger-ui) | Single-binary philosophy, in-browser "Try it out" testing |
| SDK generation | openapi-generator-cli, in-repo | Deterministic, reviewable, no CI dependency |
| Publication | Manual publish scripts | No CI secrets setup needed, publish on release |
| Route coverage | Priority-first (data, devices, alarms, commands first) | Avoid single massive PR, ship value incrementally |

---

## Section 1: OpenAPI Spec Generation

### utoipa Integration

Add to `Cargo.toml`:
```toml
[dependencies]
utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }
```

### Annotation Pattern

Each handler gets `#[utoipa::path]` with operation metadata:

```rust
#[utoipa::path(
    get,
    path = "/api/v1/data",
    params(
        ("screen" = Option<String>, Query, description = "Screen ID to filter data sources")
    ),
    responses(
        (status = 200, description = "Data source values", body = Vec<DataSourceData>),
        (status = 401, description = "Unauthorized")
    ),
    security(("api_key" = []), ("cookie_auth" = [])),
    tag = "Data"
)]
pub async fn get_data(Query(params): Query<DataQuery>, State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    // existing handler code unchanged
}
```

### Schema Types

Existing model structs get `ToSchema` derive:
```rust
#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DataSourceData {
    pub id: i64,
    pub key: String,
    pub value: f64,
    // ...
}
```

### OpenApi Builder

New file `src/api/openapi.rs`:
```rust
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Data endpoints
        crate::api::data::get_data,
        crate::api::data::get_history,
        crate::api::data::export_csv,
        // Device endpoints
        crate::api::modbus_devices::list_modbus_devices,
        // ... all annotated handlers
    ),
    components(schemas(
        crate::models::DataSourceData,
        // ... all schema types
    )),
    modifiers(&SecurityAddon),
    security(("api_key" = []), ("cookie_auth" = []))
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // Add API key security scheme
    }
}
```

### Route Registration

In `src/api/mod.rs`, merge Swagger UI into the router:
```rust
let swagger = SwaggerUi::new("/api/docs")
    .url("/api/openapi.json", ApiDoc::openapi());

let app = Router::new()
    .merge(swagger)
    // ... existing routes
```

### Priority Annotation Order

**Tier 1 (ship first):** data, history, export, devices (modbus/opcua/s7/enip), alarms, commands, auth
**Tier 2 (next):** screens, layouts, data-sources, flows, notifications, stream
**Tier 3 (last):** oee, analytics, hub, reports, recipes, rbac, compliance, security, annotations, shift-handover, video, compression, api-keys

---

## Section 2: SDK Generation & Structure

### Directory Structure

```
sdks/
├── generate.sh                  # Fetches spec, runs generator for both languages
├── python/
│   ├── voltrus/                 # Generated Python package
│   │   ├── __init__.py
│   │   ├── api_client.py
│   │   ├── configuration.py
│   │   ├── api/                 # Per-tag API classes
│   │   └── models/              # Pydantic models
│   ├── pyproject.toml           # Package: voltrus, version synced with app
│   ├── publish.sh               # twine upload to PyPI
│   └── examples/
│       ├── read_tags.py
│       ├── write_setpoint.py
│       └── subscribe_alarms.py
└── typescript/
    ├── src/
    │   ├── api/                 # Per-tag API classes
    │   └── models/              # TypeScript interfaces
    ├── package.json             # Package: voltrus, version synced with app
    ├── tsconfig.json
    ├── publish.sh               # npm publish
    └── examples/
        ├── read_tags.ts
        ├── write_setpoint.ts
        └── subscribe_alarms.ts
```

### generate.sh Flow

```bash
#!/bin/bash
# 1. Start Voltrus briefly to dump the spec (or use a dedicated --dump-spec CLI flag)
# 2. Run openapi-generator-cli for Python
# 3. Run openapi-generator-cli for TypeScript
# 4. Copy custom README and examples into generated packages
```

The primary generation path uses `voltrus --dump-openapi` (writes spec JSON to stdout without starting the server). This avoids needing a running instance for SDK generation.

### SDK Auth

Both SDKs support:
- **API Key auth:** Constructor takes `api_key`, sent as `X-API-Key` header
- **Cookie auth:** Login method calls `/api/v1/auth/login`, stores session cookie

```python
# Python example
from voltrus import VoltrusClient

client = VoltrusClient(base_url="http://192.168.1.100:3000", api_key="vt_sk_abc123")
data = client.data.get_data(screen="overview")
```

```typescript
// TypeScript example
import { VoltrusClient } from "voltrus";

const client = new VoltrusClient({ baseUrl: "http://192.168.1.100:3000", apiKey: "vt_sk_abc123" });
const data = await client.data.getData({ screen: "overview" });
```

### Example Scripts

**read_tags.py / read_tags.ts:**
- Connect to Voltrus, fetch `/api/v1/data?screen=X`, print all tag values with status

**write_setpoint.py / write_setpoint.ts:**
- Connect, POST `/api/v1/tags/{id}/write` with a value
- Poll `/api/v1/commands/{command_id}` until ack'd or failed

**subscribe_alarms.py / subscribe_alarms.ts:**
- Connect to SSE stream `/api/v1/stream`
- Filter for `alarm` event types
- Print alarm details as they arrive

---

## Section 3: Swagger UI & API Explorer

### Embedded UI

`utoipa-swagger-ui` bundles all static assets at compile time. No external files, no CDN dependency. Single-binary philosophy preserved.

### Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /api/docs` | Swagger UI (interactive API explorer) |
| `GET /api/openapi.json` | Raw OpenAPI 3.1 spec JSON |

Both endpoints are **public** (no auth required). The spec documents which endpoints require auth.

### Auth in Swagger UI

Configured security schemes:
- `api_key`: API key passed in `X-API-Key` header
- `cookie_auth`: Session cookie from `/api/v1/auth/login`

Users click "Authorize" in Swagger UI, paste their API key, and can test any endpoint live.

### Tier Enforcement Documentation

Each endpoint's description includes tier requirements:
```
Tier: Starter ($249)
```
This is informational — actual enforcement happens server-side via license checks.

---

## Section 4: Documentation & Landing Page

### SDK Documentation (docs site update)

New section in `voltrus-landing/docs.html`:
- **API Reference:** Link to `/api/docs` on a running instance
- **SDK Quickstart:** Install, configure, first request (both languages)
- **Auth Guide:** API key setup, cookie auth, security best practices
- **Examples:** The 3 example scripts with explanations

### SDK README Files

`sdks/python/README.md`:
- Install: `pip install voltrus`
- Quickstart with API key auth
- Link to full API reference (Swagger UI)
- License: MIT

`sdks/typescript/README.md`:
- Install: `npm install voltrus`
- Quickstart with API key auth
- Link to full API reference (Swagger UI)
- License: MIT

### Landing Page Update

`voltrus-landing/index.html`:
- Add "Developer SDKs" to feature list
- "Python & TypeScript SDKs auto-generated from OpenAPI spec"
- "Interactive API explorer with Swagger UI"
- Link to docs for SDK setup

### Changelog Entry

`voltrus-landing/changelog.html`:
- v0.27.0 entry with SDK + OpenAPI features

---

## Section 5: v0.26.0 Merge Coordination

v0.26.0 (other agent) adds simulation/scripting endpoints and modifies `src/api/mod.rs`. v0.27.0 adds utoipa annotations to existing handlers and also modifies `src/api/mod.rs` (Swagger UI route registration).

**Merge plan:**
1. v0.26.0 merges first (earlier version)
2. v0.27.0 rebases onto merged result
3. Conflicts in `src/api/mod.rs` are additive (both append routes) — easy resolution
4. v0.26.0's new endpoints need `#[utoipa::path]` annotations added post-merge

**Files modified by both:**
- `src/api/mod.rs` — both add routes (additive, no structural conflicts)
- `Cargo.toml` — both add dependencies (additive)

---

## Section 6: New Files & Module Structure

### Backend (Rust)

| File | Purpose |
|------|---------|
| `src/api/openapi.rs` | OpenApi builder, Swagger UI setup, security schemes |
| Modifications to `src/api/mod.rs` | Register Swagger UI routes |
| Modifications to `src/api/*.rs` (all 34 modules) | Add `#[utoipa::path]` annotations |
| Modifications to `src/models.rs` | Add `ToSchema` derives to response/request types |
| Modifications to `Cargo.toml` | Add utoipa, utoipa-swagger-ui dependencies |
| Modifications to `src/main.rs` | Add `--dump-openapi` CLI flag |

### SDKs

| File | Purpose |
|------|---------|
| `sdks/generate.sh` | SDK generation script |
| `sdks/python/` | Generated Python SDK + examples + publish script |
| `sdks/typescript/` | Generated TypeScript SDK + examples + publish script |

### Frontend / Landing

| File | Purpose |
|------|---------|
| `voltrus-landing/docs.html` | SDK quickstart, API reference section |
| `voltrus-landing/index.html` | Developer SDKs feature mention |
| `voltrus-landing/changelog.html` | v0.27.0 entry |

---

## Testing Strategy

- **Unit tests:** OpenApi spec generation produces valid JSON, all annotated paths present
- **Integration tests:** Swagger UI returns HTML, spec returns valid OpenAPI 3.1, all schemas resolve
- **SDK tests:** Python SDK tests (pytest) — connect, read data, write command. TypeScript SDK tests (jest) — same.
- **Validation:** `openapi-generator-cli validate` on generated spec
- **Version sync:** Test that SDK package version matches Voltrus binary version

---

## Implementation Order

1. Add utoipa dependencies to Cargo.toml
2. Create `src/api/openapi.rs` with OpenApi builder and Swagger UI setup
3. Add `ToSchema` derives to models
4. Annotate Tier 1 handlers (data, devices, alarms, commands, auth)
5. Register Swagger UI routes in mod.rs
6. Test spec generation and Swagger UI
7. Add `--dump-openapi` CLI flag
8. Generate Python SDK via openapi-generator-cli
9. Generate TypeScript SDK via openapi-generator-cli
10. Write example scripts (3 per language)
11. Annotate Tier 2 handlers
12. Annotate Tier 3 handlers
13. Update docs site, landing page, changelog
14. Manual publish scripts for PyPI/npm
