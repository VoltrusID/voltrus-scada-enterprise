# v0.27.0 — SDKs & OpenAPI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add OpenAPI 3.1 spec generation via utoipa, embed Swagger UI, generate Python + TypeScript SDKs with example scripts.

**Architecture:** utoipa procedural macros annotate existing Axum handlers. A single `OpenApi` struct aggregates all paths/schemas. Swagger UI is embedded via `utoipa-swagger-ui`. SDKs are generated in-repo via `openapi-generator-cli`.

**Tech Stack:** utoipa 5.x, utoipa-swagger-ui 9.x, utoipa-axum (for route collection), openapi-generator-cli, Python (PyPI), TypeScript (npm)

**Spec:** `docs/superpowers/specs/2026-05-03-sdks-openapi-design.md`

---

## File Structure

| File | Action | Purpose |
|------|--------|---------|
| `Cargo.toml` | Modify | Add utoipa, utoipa-swagger-ui, utoipa-axum deps |
| `src/api/openapi.rs` | Create | OpenApi builder, security schemes, tags |
| `src/api/mod.rs` | Modify | Register Swagger UI routes, add `mod openapi` |
| `src/api/data.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/auth.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/system.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/modbus_devices.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/opcua_devices.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/s7_devices.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/enip_devices.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/alarms.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/commands.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/compression.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/screens.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/layouts.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/data_sources.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/flows.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/notifications.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/stream.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/oee.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/analytics.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/hub.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/reports.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/recipes.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/rbac.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/compliance.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/security.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/annotations.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/shift_handover.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/video.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/api_keys.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/settings.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/shortcuts.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/push.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/user_notifications.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/license.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/api/generators.rs` | Modify | Add `#[utoipa::path]` annotations |
| `src/models.rs` | Modify | Add `ToSchema` derives to response/request types |
| `src/main.rs` | Modify | Add `--dump-openapi` CLI flag |
| `sdks/generate.sh` | Create | SDK generation script |
| `sdks/python/` | Create | Generated Python SDK + examples |
| `sdks/typescript/` | Create | Generated TypeScript SDK + examples |

---

### Task 1: Add utoipa Dependencies

**Files:**
- Modify: `Cargo.toml:26-64`

- [ ] **Step 1: Add utoipa dependencies to Cargo.toml**

Add these three lines after the `axum-extra` line (line 39) in `[dependencies]`:

```toml
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "9", features = ["axum"] }
utoipa-axum = "0.2"
```

Also add to `[dev-dependencies]`:
```toml
utoipa = { version = "5", features = ["axum_extras"] }
```

- [ ] **Step 2: Run cargo check to verify dependencies resolve**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiling/Downloading new crates, no errors

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add utoipa, utoipa-swagger-ui, utoipa-axum dependencies"
```

---

### Task 2: Create OpenApi Builder + ToSchema on Core Models

**Files:**
- Create: `src/api/openapi.rs`
- Modify: `src/models.rs` (add `ToSchema` derives)
- Modify: `src/api/mod.rs:0-1` (add `mod openapi`)

- [ ] **Step 1: Add `mod openapi;` to `src/api/mod.rs`**

Add at line 1 (before existing mod declarations):
```rust
mod openapi;
```

- [ ] **Step 2: Add `utoipa::ToSchema` derives to model structs in `src/models.rs`**

For each of the following structs, add `utoipa::ToSchema` to the derive attribute. **Do not remove existing derives.** Only add `utoipa::ToSchema` alongside existing `Serialize`/`Deserialize`/`Clone`/`Debug`.

Structs to annotate (search by name, add `utoipa::ToSchema` to their `#[derive(...)]`):

```
HealthResponse, SystemResponse, ProcessInfo, LiveResponse,
LoginResponse, BrandingSettings, Role,
DataSourceData, HistoryResponse, DataSourceSummary, TelemetryExportRow, ExportMeta, ExportResponse,
Alarm, AlarmAuditEntry, AlarmSummary, AlarmAnalyticsEntry,
Command, CommandAuditEntry,
ModbusDevice, ScannedRegister, PollerStatus,
OpcUaDevice, OpcUaSecurityPolicy, OpcUaSecurityMode,
Camera, HubSite
```

Also add these derives to any query/request structs that exist. Search for them across `src/models.rs` and `src/api/*.rs` — they may be defined locally in the API module:
```
DataQuery (likely in src/api/data.rs — search for `struct DataQuery`)
AlarmListQuery (likely in src/api/alarms.rs)
CommandQuery (likely in src/api/commands.rs)
WriteRequest (likely in src/api/commands.rs)
LoginRequest (likely in src/api/auth.rs)
```

If any of these are defined as local structs in the API module files (not in `src/models.rs`), add `utoipa::ToSchema` to their derive in that file instead.

For structs that have `#[derive(Debug, Clone, serde::Serialize)]`, change to:
```rust
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
```

For structs that have `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`, change to:
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
```

**Important:** Some structs have `pub` fields with types like `HashMap<String, String>` or `serde_json::Value`. utoipa handles these natively — no special handling needed.

- [ ] **Step 3: Create `src/api/openapi.rs`**

```rust
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication and user management"),
        (name = "Data", description = "Real-time data and history queries"),
        (name = "Devices", description = "Device management (Modbus, OPC-UA, S7, EtherNet/IP)"),
        (name = "Alarms", description = "Alarm management and analytics"),
        (name = "Commands", description = "Command acknowledgment and control"),
        (name = "System", description = "System health, info, and memory"),
        (name = "Compression", description = "Data compression stats and config"),
        (name = "Screens", description = "Dashboard screen management"),
        (name = "Layouts", description = "Screen layout management"),
        (name = "Data Sources", description = "Data source configuration"),
        (name = "Flows", description = "Data flow engine rules"),
        (name = "Notifications", description = "Notification settings and push subscriptions"),
        (name = "Stream", description = "Server-Sent Events real-time stream"),
        (name = "OEE", description = "Overall Equipment Effectiveness metrics"),
        (name = "Analytics", description = "Data aggregation and comparison"),
        (name = "Hub", description = "Multi-site hub management"),
        (name = "Reports", description = "Report generation and scheduling"),
        (name = "Recipes", description = "Recipe and parameter management"),
        (name = "RBAC", description = "Role-based access control"),
        (name = "Compliance", description = "FDA 21 CFR Part 11 compliance"),
        (name = "Security", description = "IEC 62443 security audit events"),
        (name = "Annotations", description = "Data point annotations"),
        (name = "Shift Handover", description = "Shift handover logs and reports"),
        (name = "Video", description = "RTSP camera management"),
        (name = "API Keys", description = "API key management"),
        (name = "Settings", description = "Application settings"),
        (name = "License", description = "License status"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("voltrus_token"))),
            );
        } else {
            let components = utoipa::openapi::ComponentsBuilder::new()
                .security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
                )
                .security_scheme(
                    "cookie_auth",
                    SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("voltrus_token"))),
                )
                .build();
            openapi.components = Some(components);
        }
    }
}
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check 2>&1 | tail -10`
Expected: Compiles with warnings (unused import is fine), no errors

- [ ] **Step 5: Commit**

```bash
git add src/api/openapi.rs src/api/mod.rs src/models.rs
git commit -m "feat(openapi): create ApiDoc builder with security schemes and ToSchema on models"
```

---

### Task 3: Annotate Public Routes (Auth + System + License)

**Files:**
- Modify: `src/api/system.rs`
- Modify: `src/api/auth.rs`
- Modify: `src/api/settings.rs`
- Modify: `src/api/license.rs`
- Modify: `src/api/openapi.rs` (register paths and schemas)

**Handler functions to annotate** (read each file, find the function, add `#[utoipa::path(...)]` above it):

**`src/api/system.rs`:**
- `get_health` — `get, path = "/api/v1/health", responses((status=200, body=HealthResponse)), tag="System"`
- `get_live` — `get, path = "/api/v1/live", responses((status=200, body=LiveResponse)), tag="System", security(("api_key"=[], "cookie_auth"=[]))`
- `get_system` — `get, path = "/api/v1/system", responses((status=200, body=SystemResponse)), tag="System", security(("api_key"=[], "cookie_auth"=[]))`
- `get_version` — `get, path = "/api/v1/version", responses((status=200, description="Version info")), tag="System", security(("api_key"=[], "cookie_auth"=[]))`

**`src/api/auth.rs`:**
- `login` — `post, path = "/api/v1/auth/login", request_body=LoginRequest, responses((status=200, body=LoginResponse),(status=401)), tag="Auth"`
- `refresh_token` — `post, path = "/api/v1/auth/refresh", responses((status=200, body=LoginResponse),(status=401)), tag="Auth"`
- `logout` — `post, path = "/api/v1/auth/logout", responses((status=200)), tag="Auth", security(("api_key"=[], "cookie_auth"=[]))`
- `auth_me` — `get, path = "/api/v1/auth/me", responses((status=200)), tag="Auth", security(("api_key"=[], "cookie_auth"=[]))`
- `get_csrf_token` — `get, path = "/api/v1/auth/csrf-token", responses((status=200)), tag="Auth", security(("api_key"=[], "cookie_auth"=[]))`
- `list_auth_providers` — `get, path = "/api/v1/auth/providers", responses((status=200)), tag="Auth"`
- `saml_metadata` — `post, path = "/api/v1/auth/saml/metadata", responses((status=200)), tag="Auth"`
- `saml_acs` — `post, path = "/api/v1/auth/saml/acs", responses((status=200),(status=302)), tag="Auth"`
- `ldap_login` — `post, path = "/api/v1/auth/ldap/login", responses((status=200, body=LoginResponse),(status=401)), tag="Auth"`
- `change_password` — `post, path = "/api/v1/auth/change-password", responses((status=200),(status=401)), tag="Auth", security(("api_key"=[], "cookie_auth"=[]))`
- `seed` — `post, path = "/api/v1/seed", responses((status=200),(status=409)), tag="Auth"`

**`src/api/settings.rs`:**
- `get_branding` — `get, path = "/api/v1/settings/branding", responses((status=200, body=BrandingSettings)), tag="Settings"`
- `get_alarm_cooldown` — `get, path = "/api/v1/settings/alarm-cooldown", responses((status=200)), tag="Settings", security(("api_key"=[], "cookie_auth"=[]))`
- `update_branding` — `put, path = "/api/v1/settings/branding", responses((status=200)), tag="Settings", security(("api_key"=[], "cookie_auth"=[]))`
- `update_alarm_cooldown` — `put, path = "/api/v1/settings/alarm-cooldown", responses((status=200)), tag="Settings", security(("api_key"=[], "cookie_auth"=[]))`

**`src/api/license.rs`:**
- `get_license_status` — `get, path = "/api/license/status", responses((status=200)), tag="License"`

- [ ] **Step 1: Add annotations to all handlers listed above**

For each handler function, add a `#[utoipa::path(...)]` attribute directly above the `pub async fn` line. Example for `get_health`:

```rust
#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses((status = 200, description = "Health check", body = crate::models::HealthResponse)),
    tag = "System"
)]
pub async fn get_health(State(state): State<AppState>) -> Json<HealthResponse> {
```

For handlers that use `Extension<AuthUser>` (admin-only), add security:
```rust
#[utoipa::path(
    post,
    path = "/api/v1/seed",
    responses((status = 200, description = "Seeded"), (status = 409, description = "Already seeded")),
    security(("api_key" = []), ("cookie_auth" = [])),
    tag = "Auth"
)]
pub async fn seed(...) {
```

- [ ] **Step 2: Register all paths and schemas in `src/api/openapi.rs`**

Add `paths(...)` and `components(schemas(...))` to the `#[openapi(...)]` attribute on `ApiDoc`:

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        // System
        crate::api::system::get_health,
        crate::api::system::get_live,
        crate::api::system::get_system,
        crate::api::system::get_version,
        // Auth
        crate::api::auth::login,
        crate::api::auth::refresh_token,
        crate::api::auth::logout,
        crate::api::auth::auth_me,
        crate::api::auth::get_csrf_token,
        crate::api::auth::list_auth_providers,
        crate::api::auth::saml_metadata,
        crate::api::auth::saml_acs,
        crate::api::auth::ldap_login,
        crate::api::auth::change_password,
        crate::api::auth::seed,
        // Settings
        crate::api::settings::get_branding,
        crate::api::settings::get_alarm_cooldown,
        crate::api::settings::update_branding,
        crate::api::settings::update_alarm_cooldown,
        // License
        crate::api::license::get_license_status,
    ),
    components(schemas(
        crate::models::HealthResponse,
        crate::models::SystemResponse,
        crate::models::ProcessInfo,
        crate::models::LiveResponse,
        crate::models::LoginResponse,
        crate::models::BrandingSettings,
    )),
    modifiers(&SecurityAddon),
    tags(
        // ... existing tags ...
    )
)]
pub struct ApiDoc;
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check 2>&1 | tail -10`
Expected: No errors (warnings OK)

- [ ] **Step 4: Commit**

```bash
git add src/api/system.rs src/api/auth.rs src/api/settings.rs src/api/license.rs src/api/openapi.rs
git commit -m "feat(openapi): annotate public routes (auth, system, settings, license)"
```

---

### Task 4: Annotate Data + Device Routes

**Files:**
- Modify: `src/api/data.rs`
- Modify: `src/api/modbus_devices.rs`
- Modify: `src/api/opcua_devices.rs`
- Modify: `src/api/s7_devices.rs`
- Modify: `src/api/enip_devices.rs`
- Modify: `src/api/openapi.rs` (register new paths and schemas)

**Handler functions to annotate:**

**`src/api/data.rs`:**
- `get_data` — `get, path="/api/v1/data", params(("screen"=Option<String>, Query)), responses((status=200, body=Vec<DataSourceData>)), tag="Data", security(...)`
- `get_history` — `get, path="/api/v1/history", params(("screen"=Option<String>, Query),("range"=Option<String>, Query),("interval"=Option<u64>, Query),("from"=Option<i64>, Query),("to"=Option<i64>, Query)), responses((status=200, body=HistoryResponse)), tag="Data", security(...)`
- `export_csv` — `get, path="/api/v1/export/csv", responses((status=200, description="CSV file")), tag="Data", security(...)`
- `export_data` — `get, path="/api/v1/data/export", responses((status=200, body=ExportResponse)), tag="Data", security(...)`

**`src/api/modbus_devices.rs`:**
- `list_modbus_devices` — `get, path="/api/v1/modbus-devices", responses((status=200)), tag="Devices", security(...)`
- `create_modbus_device` — `post, path="/api/v1/modbus-devices", responses((status=201),(status=400)), tag="Devices", security(...)`
- `update_modbus_device` — `put, path="/api/v1/modbus-devices/{id}", params(("id"=String, Path)), responses((status=200),(status=404)), tag="Devices", security(...)`
- `delete_modbus_device` — `delete, path="/api/v1/modbus-devices/{id}", params(("id"=String, Path)), responses((status=200),(status=404)), tag="Devices", security(...)`
- `scan_modbus_registers` — `post, path="/api/v1/modbus-devices/scan", responses((status=200, body=Vec<ScannedRegister>)), tag="Devices", security(...)`
- `scan_network` — `post, path="/api/v1/modbus-devices/scan-network", responses((status=200)), tag="Devices", security(...)`
- `replace_registers` — `put, path="/api/v1/modbus-devices/{id}/registers", params(("id"=String, Path)), responses((status=200)), tag="Devices", security(...)`
- `restart_poller` — `post, path="/api/v1/modbus-devices/{id}/restart", params(("id"=String, Path)), responses((status=200)), tag="Devices", security(...)`
- `get_poller_statuses` — `get, path="/api/v1/modbus-devices/status", responses((status=200, body=Vec<PollerStatus>)), tag="Devices", security(...)`

**`src/api/opcua_devices.rs`:**
- `list_devices` — `get, path="/api/v1/opcua/devices", responses((status=200)), tag="Devices", security(...)`
- `create_device` — `post, path="/api/v1/opcua/devices", responses((status=201),(status=400)), tag="Devices", security(...)`
- `update_device` — `put, path="/api/v1/opcua/devices/{id}", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Devices", security(...)`
- `delete_device` — `delete, path="/api/v1/opcua/devices/{id}", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Devices", security(...)`
- `list_nodes` — `get, path="/api/v1/opcua/devices/{id}/nodes", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `add_node` — `post, path="/api/v1/opcua/devices/{id}/nodes", params(("id"=i64, Path)), responses((status=201)), tag="Devices", security(...)`
- `remove_node` — `delete, path="/api/v1/opcua/devices/{id}/nodes/{node_id}", params(("id"=i64, Path),("node_id"=String, Path)), responses((status=200)), tag="Devices", security(...)`
- `browse_device` — `post, path="/api/v1/opcua/devices/{id}/browse", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `start_device` — `post, path="/api/v1/opcua/devices/{id}/start", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `stop_device` — `post, path="/api/v1/opcua/devices/{id}/stop", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`

**`src/api/s7_devices.rs`:**
- `list_devices` — `get, path="/api/v1/s7/devices", responses((status=200)), tag="Devices", security(...)`
- `create_device` — `post, path="/api/v1/s7/devices", responses((status=201)), tag="Devices", security(...)`
- `update_device` — `put, path="/api/v1/s7/devices/{id}", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Devices", security(...)`
- `delete_device` — `delete, path="/api/v1/s7/devices/{id}", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Devices", security(...)`
- `list_tags` — `get, path="/api/v1/s7/devices/{id}/tags", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `add_tag` — `post, path="/api/v1/s7/devices/{id}/tags", params(("id"=i64, Path)), responses((status=201)), tag="Devices", security(...)`
- `remove_tag` — `delete, path="/api/v1/s7/devices/{id}/tags/{tag_id}", params(("id"=i64, Path),("tag_id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `browse_device` — `post, path="/api/v1/s7/devices/{id}/browse", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `start_device` — `post, path="/api/v1/s7/devices/{id}/start", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `stop_device` — `post, path="/api/v1/s7/devices/{id}/stop", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`

**`src/api/enip_devices.rs`:**
- `list_devices` — `get, path="/api/v1/enip/devices", responses((status=200)), tag="Devices", security(...)`
- `create_device` — `post, path="/api/v1/enip/devices", responses((status=201)), tag="Devices", security(...)`
- `update_device` — `put, path="/api/v1/enip/devices/{id}", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Devices", security(...)`
- `delete_device` — `delete, path="/api/v1/enip/devices/{id}", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Devices", security(...)`
- `list_tags` — `get, path="/api/v1/enip/devices/{id}/tags", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `add_tag` — `post, path="/api/v1/enip/devices/{id}/tags", params(("id"=i64, Path)), responses((status=201)), tag="Devices", security(...)`
- `remove_tag` — `delete, path="/api/v1/enip/devices/{id}/tags/{tag_id}", params(("id"=i64, Path),("tag_id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `browse_device` — `post, path="/api/v1/enip/devices/{id}/browse", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `start_device` — `post, path="/api/v1/enip/devices/{id}/start", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`
- `stop_device` — `post, path="/api/v1/enip/devices/{id}/stop", params(("id"=i64, Path)), responses((status=200)), tag="Devices", security(...)`

- [ ] **Step 1: Add annotations to all handlers listed above**

- [ ] **Step 2: Register new paths and schemas in `src/api/openapi.rs`**

Add to the `paths(...)` list:
```rust
    // Data
    crate::api::data::get_data,
    crate::api::data::get_history,
    crate::api::data::export_csv,
    crate::api::data::export_data,
    // Modbus
    crate::api::modbus_devices::list_modbus_devices,
    crate::api::modbus_devices::create_modbus_device,
    crate::api::modbus_devices::update_modbus_device,
    crate::api::modbus_devices::delete_modbus_device,
    crate::api::modbus_devices::scan_modbus_registers,
    crate::api::modbus_devices::scan_network,
    crate::api::modbus_devices::replace_registers,
    crate::api::modbus_devices::restart_poller,
    crate::api::modbus_devices::get_poller_statuses,
    // OPC-UA
    crate::api::opcua_devices::list_devices,
    crate::api::opcua_devices::create_device,
    crate::api::opcua_devices::update_device,
    crate::api::opcua_devices::delete_device,
    crate::api::opcua_devices::list_nodes,
    crate::api::opcua_devices::add_node,
    crate::api::opcua_devices::remove_node,
    crate::api::opcua_devices::browse_device,
    crate::api::opcua_devices::start_device,
    crate::api::opcua_devices::stop_device,
    // S7
    crate::api::s7_devices::list_devices,
    crate::api::s7_devices::create_device,
    crate::api::s7_devices::update_device,
    crate::api::s7_devices::delete_device,
    crate::api::s7_devices::list_tags,
    crate::api::s7_devices::add_tag,
    crate::api::s7_devices::remove_tag,
    crate::api::s7_devices::browse_device,
    crate::api::s7_devices::start_device,
    crate::api::s7_devices::stop_device,
    // EtherNet/IP
    crate::api::enip_devices::list_devices,
    crate::api::enip_devices::create_device,
    crate::api::enip_devices::update_device,
    crate::api::enip_devices::delete_device,
    crate::api::enip_devices::list_tags,
    crate::api::enip_devices::add_tag,
    crate::api::enip_devices::remove_tag,
    crate::api::enip_devices::browse_device,
    crate::api::enip_devices::start_device,
    crate::api::enip_devices::stop_device,
```

Add to the `components(schemas(...))` list:
```rust
    crate::models::DataSourceData,
    crate::models::HistoryResponse,
    crate::models::DataSourceSummary,
    crate::models::ExportResponse,
    crate::models::ExportMeta,
    crate::models::TelemetryExportRow,
    crate::models::ModbusDevice,
    crate::models::ScannedRegister,
    crate::models::PollerStatus,
    crate::models::OpcUaDevice,
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check 2>&1 | tail -10`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src/api/data.rs src/api/modbus_devices.rs src/api/opcua_devices.rs src/api/s7_devices.rs src/api/enip_devices.rs src/api/openapi.rs
git commit -m "feat(openapi): annotate data and device routes (modbus, opcua, s7, enip)"
```

---

### Task 5: Annotate Alarm + Command Routes

**Files:**
- Modify: `src/api/alarms.rs`
- Modify: `src/api/commands.rs`
- Modify: `src/api/openapi.rs`

**Handlers to annotate:**

**`src/api/alarms.rs`:**
- `list_alarms` — `get, path="/api/v1/alarms", params(("severity"=Option<String>, Query),("state"=Option<String>, Query),("limit"=Option<u32>, Query),("offset"=Option<u32>, Query)), responses((status=200, body=Vec<Alarm>)), tag="Alarms", security(...)`
- `acknowledge_all` — `post, path="/api/v1/alarms", responses((status=200)), tag="Alarms", security(...)`
- `active_alarm_count` — `get, path="/api/v1/alarms/active/count", responses((status=200)), tag="Alarms", security(...)`
- `acknowledge_alarm` — `post, path="/api/v1/alarms/{id}/acknowledge", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Alarms", security(...)`
- `shelf_alarm` — `post, path="/api/v1/alarms/{id}/shelf", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Alarms", security(...)`
- `unshelf_alarm` — `post, path="/api/v1/alarms/{id}/unshelf", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Alarms", security(...)`
- `list_alarm_audit` — `get, path="/api/v1/alarms/audit", responses((status=200, body=Vec<AlarmAuditEntry>)), tag="Alarms", security(...)`
- `list_alarm_analytics` — `get, path="/api/v1/alarms/analytics", responses((status=200, body=Vec<AlarmAnalyticsEntry>)), tag="Alarms", security(...)`
- `get_alarm_summary` — `get, path="/api/v1/alarms/summary", responses((status=200, body=AlarmSummary)), tag="Alarms", security(...)`

**`src/api/commands.rs`:**
- `list_commands` — `get, path="/api/v1/commands", params(("tag"=Option<String>, Query),("status"=Option<String>, Query),("limit"=Option<u32>, Query)), responses((status=200, body=Vec<Command>)), tag="Commands", security(...)`
- `get_command_status` — `get, path="/api/v1/commands/{command_id}", params(("command_id"=String, Path)), responses((status=200, body=Command),(status=404)), tag="Commands", security(...)`
- `get_command_audit` — `get, path="/api/v1/commands/{command_id}/audit", params(("command_id"=String, Path)), responses((status=200, body=Vec<CommandAuditEntry>)), tag="Commands", security(...)`
- `write_tag` — `post, path="/api/v1/tags/{id}/write", params(("id"=i64, Path)), request_body=WriteRequest, responses((status=202, description="Command accepted"),(status=403),(status=404)), tag="Commands", security(...)`
- `cancel_command` — `post, path="/api/v1/commands/{command_id}/cancel", params(("command_id"=String, Path)), responses((status=200),(status=404)), tag="Commands", security(...)`
- `update_write_policy` — `patch, path="/api/v1/data-sources/{id}/write-policy", params(("id"=i64, Path)), responses((status=200),(status=404)), tag="Commands", security(...)`

- [ ] **Step 1: Add annotations to all handlers**

- [ ] **Step 2: Register paths and schemas in openapi.rs**

Add to paths:
```rust
    // Alarms
    crate::api::alarms::list_alarms,
    crate::api::alarms::acknowledge_all,
    crate::api::alarms::active_alarm_count,
    crate::api::alarms::acknowledge_alarm,
    crate::api::alarms::shelf_alarm,
    crate::api::alarms::unshelf_alarm,
    crate::api::alarms::list_alarm_audit,
    crate::api::alarms::list_alarm_analytics,
    crate::api::alarms::get_alarm_summary,
    // Commands
    crate::api::commands::list_commands,
    crate::api::commands::get_command_status,
    crate::api::commands::get_command_audit,
    crate::api::commands::write_tag,
    crate::api::commands::cancel_command,
    crate::api::commands::update_write_policy,
```

Add to schemas:
```rust
    crate::models::Alarm,
    crate::models::AlarmAuditEntry,
    crate::models::AlarmSummary,
    crate::models::AlarmAnalyticsEntry,
    crate::models::Command,
    crate::models::CommandAuditEntry,
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check 2>&1 | tail -10`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src/api/alarms.rs src/api/commands.rs src/api/openapi.rs
git commit -m "feat(openapi): annotate alarm and command routes"
```

---

### Task 6: Wire Up Swagger UI + --dump-openapi CLI + Tests

**Files:**
- Modify: `src/api/mod.rs` (merge Swagger UI into router)
- Modify: `src/main.rs` (add `--dump-openapi` CLI flag)
- Create: test in `src/api/openapi.rs` (unit test for spec generation)

- [ ] **Step 1: Add Swagger UI to router in `src/api/mod.rs`**

Add import at top of file:
```rust
use utoipa_swagger_ui::SwaggerUi;
```

Add `mod openapi;` is already done. Now in the `create_router` function, **before** the `let api_router = Router::new()` line, add:

```rust
    let swagger = SwaggerUi::new("/api/docs")
        .url("/api/openapi.json", openapi::ApiDoc::openapi());
```

Then merge swagger into the api_router. After `let api_router = Router::new()`:
```rust
    let api_router = Router::new()
        .merge(swagger)
        .merge(public)
```

- [ ] **Step 2: Add `--dump-openapi` CLI flag in `src/main.rs`**

Find the CLI args match block. Add a new arm before the `"db"` match arm:

```rust
        "--dump-openapi" => {
            let spec = crate::api::openapi::ApiDoc::openapi();
            println!("{}", serde_json::to_string_pretty(&spec)?);
            return Ok(());
        }
```

- [ ] **Step 3: Add unit test for spec generation in `src/api/openapi.rs`**

Add at bottom of file:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generates_valid_json() {
        let spec = ApiDoc::openapi();
        let json = serde_json::to_string_pretty(&spec)
            .expect("OpenAPI spec should serialize to JSON");
        assert!(!json.is_empty());
        assert!(json.contains("\"openapi\""));
        assert!(json.contains("\"paths\""));
        assert!(json.contains("\"components\""));
        assert!(json.contains("\"securitySchemes\""));
        assert!(json.contains("\"api_key\""));
        assert!(json.contains("\"cookie_auth\""));
    }

    #[test]
    fn test_openapi_spec_has_tags() {
        let spec = ApiDoc::openapi();
        let tags = spec.tags.as_ref().expect("should have tags");
        let tag_names: Vec<&str> = tags.iter().map(|t| t.name.as_str()).collect();
        assert!(tag_names.contains(&"Auth"));
        assert!(tag_names.contains(&"Data"));
        assert!(tag_names.contains(&"Devices"));
        assert!(tag_names.contains(&"Alarms"));
        assert!(tag_names.contains(&"Commands"));
        assert!(tag_names.contains(&"System"));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test openapi 2>&1`
Expected: 2 tests pass

- [ ] **Step 5: Build and test --dump-openapi**

Run: `cargo build --release 2>&1 | tail -3 && ./target/release/voltrus-lite-rust-service --dump-openapi | head -5`
Expected: Valid JSON starting with `{"openapi": "3.1.0", ...}`

- [ ] **Step 6: Commit**

```bash
git add src/api/mod.rs src/api/openapi.rs src/main.rs
git commit -m "feat(openapi): wire Swagger UI into router, add --dump-openapi CLI, add spec tests"
```

---

### Task 7: Annotate Tier 2 Routes (Screens, Layouts, Data Sources, Flows, Notifications, Stream)

**Files:**
- Modify: `src/api/screens.rs`
- Modify: `src/api/layouts.rs`
- Modify: `src/api/data_sources.rs`
- Modify: `src/api/flows.rs`
- Modify: `src/api/notifications.rs`
- Modify: `src/api/stream.rs`
- Modify: `src/api/push.rs`
- Modify: `src/api/shortcuts.rs`
- Modify: `src/api/api_keys.rs`
- Modify: `src/api/user_notifications.rs`
- Modify: `src/api/openapi.rs`

**Pattern for each file:** Read the file, find all `pub async fn` handler functions, add `#[utoipa::path(...)]` annotations above each one matching the route from `src/api/mod.rs`. All protected/admin routes get `security(("api_key" = []), ("cookie_auth" = []))`.

**Handlers to annotate:**

**`src/api/screens.rs`:**
- `list_screens` — get, path="/api/v1/screens", tag="Screens"
- `create_screen` — post, path="/api/v1/screens", tag="Screens"
- `update_screen` — put, path="/api/v1/screens/{id}", params(("id"=i64, Path)), tag="Screens"
- `delete_screen` — delete, path="/api/v1/screens/{id}", params(("id"=i64, Path)), tag="Screens"
- `list_screen_sensors` — get, path="/api/v1/screens/{id}/data-sources", params(("id"=i64, Path)), tag="Screens"
- `add_screen_sensor` — post, path="/api/v1/screens/{id}/data-sources", params(("id"=i64, Path)), tag="Screens"
- `update_screen_sensor` — put, path="/api/v1/screens/{id}/data-sources/{sensor_id}", params(("id"=i64, Path),("sensor_id"=i64, Path)), tag="Screens"
- `remove_screen_sensor` — delete, path="/api/v1/screens/{id}/data-sources/{sensor_id}", params(("id"=i64, Path),("sensor_id"=i64, Path)), tag="Screens"

**`src/api/layouts.rs`:**
- `get_layouts` — get, path="/api/v1/screens/{id}/layouts", params(("id"=i64, Path)), tag="Layouts"
- `save_layouts` — put, path="/api/v1/screens/{id}/layouts", params(("id"=i64, Path)), tag="Layouts"
- `get_twin_layouts` — get, path="/api/v1/screens/{id}/twin-layouts", params(("id"=i64, Path)), tag="Layouts"
- `save_twin_layouts` — put, path="/api/v1/screens/{id}/twin-layouts", params(("id"=i64, Path)), tag="Layouts"
- `get_twin_connections` — get, path="/api/v1/screens/{id}/twin-connections", params(("id"=i64, Path)), tag="Layouts"
- `save_twin_connections` — put, path="/api/v1/screens/{id}/twin-connections", params(("id"=i64, Path)), tag="Layouts"
- `save_twin_camera` — put, path="/api/v1/screens/{id}/twin-camera", params(("id"=i64, Path)), tag="Layouts"
- `get_canvas_layouts` — get, path="/api/v1/screens/{id}/canvas-layouts", params(("id"=i64, Path)), tag="Layouts"
- `save_canvas_layouts` — put, path="/api/v1/screens/{id}/canvas-layouts", params(("id"=i64, Path)), tag="Layouts"
- `get_scada_equipment` — get, path="/api/v1/screens/{id}/scada-equipment", params(("id"=i64, Path)), tag="Layouts"
- `save_scada_equipment` — put, path="/api/v1/screens/{id}/scada-equipment", params(("id"=i64, Path)), tag="Layouts"
- `get_scada_connections` — get, path="/api/v1/screens/{id}/scada-connections", params(("id"=i64, Path)), tag="Layouts"
- `save_scada_connections` — put, path="/api/v1/screens/{id}/scada-connections", params(("id"=i64, Path)), tag="Layouts"

**`src/api/data_sources.rs`:**
- `list_data_sources` — get, path="/api/v1/data-sources", tag="Data Sources"
- `create_data_source` — post, path="/api/v1/data-sources", tag="Data Sources"
- `update_data_source` — put, path="/api/v1/data-sources/{id}", params(("id"=i64, Path)), tag="Data Sources"
- `delete_data_source` — delete, path="/api/v1/data-sources/{id}", params(("id"=i64, Path)), tag="Data Sources"
- `reorder_data_sources` — patch, path="/api/v1/data-sources/reorder", tag="Data Sources"

**`src/api/flows.rs`:**
- `list_flows` — get, path="/api/v1/flows", tag="Flows"
- `create_flow` — post, path="/api/v1/flows", tag="Flows"
- `get_flow` — get, path="/api/v1/flows/{id}", params(("id"=i64, Path)), tag="Flows"
- `update_flow` — put, path="/api/v1/flows/{id}", params(("id"=i64, Path)), tag="Flows"
- `delete_flow` — delete, path="/api/v1/flows/{id}", params(("id"=i64, Path)), tag="Flows"
- `start_flow` — post, path="/api/v1/flows/{id}/start", params(("id"=i64, Path)), tag="Flows"
- `stop_flow` — post, path="/api/v1/flows/{id}/stop", params(("id"=i64, Path)), tag="Flows"
- `test_flow` — post, path="/api/v1/flows/{id}/test", params(("id"=i64, Path)), tag="Flows"
- `list_flow_runs` — get, path="/api/v1/flows/{id}/runs", params(("id"=i64, Path)), tag="Flows"

**`src/api/notifications.rs`:**
- `list_notification_settings` — get, path="/api/v1/notifications/settings", tag="Notifications"
- `create_notification_setting` — post, path="/api/v1/notifications/settings", tag="Notifications"
- `update_notification_setting` — put, path="/api/v1/notifications/settings/{id}", params(("id"=i64, Path)), tag="Notifications"
- `delete_notification_setting` — delete, path="/api/v1/notifications/settings/{id}", params(("id"=i64, Path)), tag="Notifications"
- `test_notification` — post, path="/api/v1/notifications/test", tag="Notifications"

**`src/api/push.rs`:**
- `subscribe` — post, path="/api/v1/push/subscribe", tag="Notifications"
- `unsubscribe` — delete, path="/api/v1/push/subscribe", tag="Notifications"
- `vapid_public_key` — get, path="/api/v1/push/vapid-key", tag="Notifications"

**`src/api/stream.rs`:**
- `stream` — get, path="/api/v1/stream", responses((status=200, description="SSE stream")), tag="Stream", security(...)

**`src/api/shortcuts.rs`:**
- `get_shortcuts` — get, path="/api/v1/shortcuts", tag="Settings"
- `save_shortcuts` — put, path="/api/v1/shortcuts", tag="Settings"
- `get_nav_shortcuts` — get, path="/api/v1/nav-shortcuts", tag="Settings"
- `save_nav_shortcuts` — put, path="/api/v1/nav-shortcuts", tag="Settings"

**`src/api/api_keys.rs`:**
- `create_api_key` — post, path="/api/v1/api-keys", tag="API Keys"
- `list_api_keys` — get, path="/api/v1/api-keys", tag="API Keys"
- `revoke_api_key` — delete, path="/api/v1/api-keys/{id}", params(("id"=i64, Path)), tag="API Keys"

**`src/api/user_notifications.rs`:**
- `list_notifications` — get, path="/api/v1/user-notifications", tag="Notifications"
- `mark_read` — put, path="/api/v1/user-notifications/{id}/read", params(("id"=i64, Path)), tag="Notifications"
- `dismiss_notification` — delete, path="/api/v1/user-notifications/{id}", params(("id"=i64, Path)), tag="Notifications"

- [ ] **Step 1: Add annotations to all handlers listed above**

- [ ] **Step 2: Register all new paths in `src/api/openapi.rs`**

Add all handler paths to the `paths(...)` list in the `#[openapi(...)]` attribute.

- [ ] **Step 3: Run cargo check**

Run: `cargo check 2>&1 | tail -10`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src/api/screens.rs src/api/layouts.rs src/api/data_sources.rs src/api/flows.rs src/api/notifications.rs src/api/stream.rs src/api/push.rs src/api/shortcuts.rs src/api/api_keys.rs src/api/user_notifications.rs src/api/openapi.rs
git commit -m "feat(openapi): annotate Tier 2 routes (screens, layouts, data-sources, flows, notifications)"
```

---

### Task 8: Annotate Tier 3 Routes (Remaining Modules)

**Files:**
- Modify: `src/api/oee.rs`
- Modify: `src/api/analytics.rs`
- Modify: `src/api/hub.rs`
- Modify: `src/api/reports.rs`
- Modify: `src/api/recipes.rs`
- Modify: `src/api/rbac.rs`
- Modify: `src/api/compliance.rs`
- Modify: `src/api/security.rs`
- Modify: `src/api/annotations.rs`
- Modify: `src/api/shift_handover.rs`
- Modify: `src/api/video.rs`
- Modify: `src/api/compression.rs`
- Modify: `src/api/openapi.rs`

**Pattern:** Same as Task 7. Read each file, find `pub async fn` handlers, match to routes from `mod.rs`, add `#[utoipa::path(...)]`. All get `security(...)` unless they're public.

**Handlers to annotate:**

**`src/api/oee.rs`:** `get_oee_metrics`, `list_downtime`, `create_downtime`, `close_downtime`, `list_shifts`, `create_shift`, `delete_shift` — tag="OEE"

**`src/api/analytics.rs`:** `get_aggregate`, `get_compare` — tag="Analytics"

**`src/api/hub.rs`:** `list_sites`, `create_site`, `update_site`, `delete_site`, `start_site_connection`, `stop_site_connection`, `get_hub_status` — tag="Hub"

**`src/api/reports.rs`:** `list_reports`, `create_report`, `update_report`, `delete_report`, `generate_report` — tag="Reports"

**`src/api/recipes.rs`:** `list_recipes`, `create_recipe`, `get_recipe`, `update_recipe`, `delete_recipe`, `load_recipe` — tag="Recipes"

**`src/api/rbac.rs`:** `list_roles`, `create_role`, `update_role`, `delete_role`, `assign_user_role` — tag="RBAC"

**`src/api/compliance.rs`:** `get_compliance_config`, `update_compliance_config`, `create_electronic_signature`, `list_electronic_signatures` — tag="Compliance"

**`src/api/security.rs`:** `list_security_events`, `export_security_csv`, `get_security_thresholds`, `update_security_thresholds` — tag="Security"

**`src/api/annotations.rs`:** `list_annotations`, `create_annotation`, `get_annotation`, `update_annotation`, `delete_annotation` — tag="Annotations"

**`src/api/shift_handover.rs`:** `list_shift_handovers`, `create_shift_handover`, `get_shift_handover`, `auto_shift_report` — tag="Shift Handover"

**`src/api/video.rs`:** `list_cameras`, `create_camera`, `update_camera`, `delete_camera`, `trigger_snapshot`, `list_snapshots`, `delete_snapshot`, `get_stream_info` — tag="Video"

**`src/api/compression.rs`:** `get_compression_stats`, `get_compression_config`, `get_system_memory` — tag="Compression"

- [ ] **Step 1: Add annotations to all handlers listed above**

- [ ] **Step 2: Register all new paths and schemas in `src/api/openapi.rs`**

Add all handler paths to `paths(...)` and any new model schemas to `components(schemas(...))`.

- [ ] **Step 3: Run full test suite**

Run: `cargo test 2>&1 | tail -5`
Expected: All tests pass (including the 2 openapi tests from Task 6)

- [ ] **Step 4: Commit**

```bash
git add src/api/oee.rs src/api/analytics.rs src/api/hub.rs src/api/reports.rs src/api/recipes.rs src/api/rbac.rs src/api/compliance.rs src/api/security.rs src/api/annotations.rs src/api/shift_handover.rs src/api/video.rs src/api/compression.rs src/api/openapi.rs
git commit -m "feat(openapi): annotate Tier 3 routes (oee, analytics, hub, reports, recipes, rbac, etc.)"
```

---

### Task 9: SDK Generation Infrastructure

**Files:**
- Create: `sdks/generate.sh`
- Create: `sdks/.openapi-generator-ignore` (Python)
- Create: `sdks/.openapi-generator-ignore` (TypeScript)

- [ ] **Step 1: Create `sdks/generate.sh`**

```bash
#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SPEC_FILE="$SCRIPT_DIR/openapi.json"
VOLTRUS_BIN="${VOLTRUS_BIN:-cargo run --bin voltrus-lite-rust-service -- --dump-openapi}"

echo "Generating OpenAPI spec..."
$VOLTRUS_BIN > "$SPEC_FILE"
echo "Spec written to $SPEC_FILE ($(wc -c < "$SPEC_FILE") bytes)"

# Generate Python SDK
echo ""
echo "Generating Python SDK..."
docker run --rm -v "$SCRIPT_DIR:/sdks" \
    openapitools/openapi-generator-cli:v7.12.0 \
    generate \
    -i /sdks/openapi.json \
    -g python \
    -o /sdks/python \
    --package-name voltrus \
    --additional-properties=generateSourceCodeOnly=true,library=urllib3

# Copy custom files back
echo "Python SDK generated in sdks/python/"

# Generate TypeScript SDK
echo ""
echo "Generating TypeScript SDK..."
docker run --rm -v "$SCRIPT_DIR:/sdks" \
    openapitools/openapi-generator-cli:v7.12.0 \
    generate \
    -i /sdks/openapi.json \
    -g typescript-axios \
    -o /sdks/typescript \
    --additional-properties=supportsES6=true,npmName=voltrus,withInterfaces=true

echo "TypeScript SDK generated in sdks/typescript/"
echo ""
echo "Done! Run 'cd sdks/python && python -m pip install -e .' to install Python SDK."
echo "Run 'cd sdks/typescript && npm install' to install TypeScript SDK."
```

Make it executable: `chmod +x sdks/generate.sh`

- [ ] **Step 2: Create Python SDK packaging files**

Create `sdks/python/pyproject.toml`:
```toml
[build-system]
requires = ["setuptools>=68.0", "wheel"]
build-backend = "setuptools.backends._legacy:_Backend"

[project]
name = "voltrus"
version = "0.27.0"
description = "Official Python SDK for Voltrus SCADA"
readme = "README.md"
license = "MIT"
requires-python = ">=3.9"
dependencies = ["urllib3>=2.0", "pydantic>=2.0"]

[project.urls]
Homepage = "https://voltrus.com"
Documentation = "https://voltrus.com/docs"
Repository = "https://github.com/dzulfikar08/voltrus"
```

Create `sdks/python/publish.sh`:
```bash
#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
python -m build
twine upload dist/*
```

Create `sdks/python/README.md`:
```markdown
# Voltrus Python SDK

Official Python SDK for the Voltrus SCADA system.

## Install

```bash
pip install voltrus
```

## Quick Start

```python
from voltrus import ApiClient, Configuration
from voltrus.api.data_api import DataApi
from voltrus.api.alarms_api import AlarmsApi

config = Configuration(host="http://192.168.1.100:3000")
config.api_key["api_key"] = "vt_sk_abc123"

with ApiClient(config) as client:
    data = DataApi(client).get_data(screen="overview")
    for ds in data:
        print(f"{ds.key}: {ds.value}")

    alarms = AlarmsApi(client).list_alarms(state="active")
    print(f"Active alarms: {len(alarms)}")
```

## License

MIT
```

- [ ] **Step 3: Create TypeScript SDK packaging files**

Create `sdks/typescript/package.json`:
```json
{
  "name": "voltrus",
  "version": "0.27.0",
  "description": "Official TypeScript SDK for Voltrus SCADA",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc",
    "prepublishOnly": "npm run build"
  },
  "license": "MIT",
  "dependencies": {
    "axios": "^1.6.0"
  },
  "devDependencies": {
    "typescript": "^5.3.0"
  },
  "homepage": "https://voltrus.com",
  "repository": "https://github.com/dzulfikar08/voltrus"
}
```

Create `sdks/typescript/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "declaration": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "examples"]
}
```

Create `sdks/typescript/publish.sh`:
```bash
#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
npm run build
npm publish
```

Create `sdks/typescript/README.md`:
```markdown
# Voltrus TypeScript SDK

Official TypeScript SDK for the Voltrus SCADA system.

## Install

```bash
npm install voltrus
```

## Quick Start

```typescript
import { Configuration, DataApi, AlarmsApi } from "voltrus";

const config = new Configuration({
  basePath: "http://192.168.1.100:3000",
  apiKey: "vt_sk_abc123",
});

const dataApi = new DataApi(config);
const data = await dataApi.getData({ screen: "overview" });
data.forEach((ds) => console.log(`${ds.key}: ${ds.value}`));

const alarmsApi = new AlarmsApi(config);
const alarms = await alarmsApi.listAlarms({ state: "active" });
console.log(`Active alarms: ${alarms.length}`);
```

## License

MIT
```

- [ ] **Step 4: Commit**

```bash
git add sdks/
git commit -m "feat(sdk): add generation script, Python/TypeScript packaging, and READMEs"
```

---

### Task 10: SDK Example Scripts

**Files:**
- Create: `sdks/python/examples/read_tags.py`
- Create: `sdks/python/examples/write_setpoint.py`
- Create: `sdks/python/examples/subscribe_alarms.py`
- Create: `sdks/typescript/examples/read_tags.ts`
- Create: `sdks/typescript/examples/write_setpoint.ts`
- Create: `sdks/typescript/examples/subscribe_alarms.ts`

- [ ] **Step 1: Create Python example — read_tags.py**

```python
"""Read tag values from Voltrus SCADA."""
import sys
from voltrus import ApiClient, Configuration
from voltrus.api.data_api import DataApi

def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:3000"
    api_key = sys.argv[2] if len(sys.argv) > 2 else ""

    config = Configuration(host=host)
    if api_key:
        config.api_key["api_key"] = api_key

    with ApiClient(config) as client:
        data = DataApi(client).get_data(screen="overview")
        for ds in data:
            status_icon = {"normal": "OK", "warning": "WARN", "critical": "CRIT", "offline": "OFF"}.get(ds.status, "?")
            print(f"[{status_icon}] {ds.key} = {ds.value:.2f} (warn={ds.warn}, crit={ds.crit})")

if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Create Python example — write_setpoint.py**

```python
"""Write a setpoint to a tag and track command acknowledgment."""
import sys
import time
from voltrus import ApiClient, Configuration
from voltrus.api.commands_api import CommandsApi

def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:3000"
    api_key = sys.argv[2] if len(sys.argv) > 2 else ""
    tag_id = int(sys.argv[3]) if len(sys.argv) > 3 else 1
    value = sys.argv[4] if len(sys.argv) > 4 else "1"

    config = Configuration(host=host)
    if api_key:
        config.api_key["api_key"] = api_key

    with ApiClient(config) as client:
        api = CommandsApi(client)
        result = api.write_tag(tag_id, {"value": value})
        command_id = result.get("command_id")
        print(f"Command submitted: {command_id}")

        for _ in range(30):
            status = api.get_command_status(command_id)
            state = status.status
            print(f"  Status: {state}")
            if state in ("acked", "resolved"):
                print("Command acknowledged!")
                return
            if state == "failed":
                print(f"Command failed: {status.error_message}")
                return
            time.sleep(1)

        print("Timeout waiting for acknowledgment")

if __name__ == "__main__":
    main()
```

- [ ] **Step 3: Create Python example — subscribe_alarms.py**

```python
"""Subscribe to real-time alarm events via SSE."""
import sys
import json
import urllib.request

def main():
    host = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:3000"
    api_key = sys.argv[2] if len(sys.argv) > 2 else ""
    url = f"{host}/api/v1/stream"

    req = urllib.request.Request(url)
    if api_key:
        req.add_header("X-API-Key", api_key)

    print(f"Connecting to {url}...")
    with urllib.request.urlopen(req) as response:
        for line in response:
            line = line.decode("utf-8").strip()
            if line.startswith("data:"):
                payload = line[5:].strip()
                try:
                    event = json.loads(payload)
                    if event.get("type") == "alarm":
                        alarm = event.get("data", {})
                        print(f"[ALARM] {alarm.get('severity', '?').upper()}: {alarm.get('message', 'N/A')}")
                except json.JSONDecodeError:
                    pass

if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Create TypeScript example — read_tags.ts**

```typescript
import { Configuration, DataApi } from "voltrus";

const host = process.argv[2] || "http://localhost:3000";
const apiKey = process.argv[3] || "";

const config = new Configuration({
  basePath: host,
  apiKey,
});

async function main() {
  const api = new DataApi(config);
  const data = await api.getData({ screen: "overview" });

  for (const ds of data) {
    const icon = { normal: "OK", warning: "WARN", critical: "CRIT", offline: "OFF" }[ds.status] || "?";
    console.log(`[${icon}] ${ds.key} = ${ds.value.toFixed(2)} (warn=${ds.warn}, crit=${ds.crit})`);
  }
}

main().catch(console.error);
```

- [ ] **Step 5: Create TypeScript example — write_setpoint.ts**

```typescript
import { Configuration, CommandsApi } from "voltrus";

const host = process.argv[2] || "http://localhost:3000";
const apiKey = process.argv[3] || "";
const tagId = parseInt(process.argv[4] || "1");
const value = process.argv[5] || "1";

const config = new Configuration({
  basePath: host,
  apiKey,
});

async function main() {
  const api = new CommandsApi(config);
  const result = await api.writeTag(tagId, { value });
  const commandId = (result.data as any).command_id;
  console.log(`Command submitted: ${commandId}`);

  for (let i = 0; i < 30; i++) {
    const status = await api.getCommandStatus(commandId);
    const state = (status.data as any).status;
    console.log(`  Status: ${state}`);

    if (state === "acked" || state === "resolved") {
      console.log("Command acknowledged!");
      return;
    }
    if (state === "failed") {
      console.log(`Command failed: ${(status.data as any).error_message}`);
      return;
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  console.log("Timeout waiting for acknowledgment");
}

main().catch(console.error);
```

- [ ] **Step 6: Create TypeScript example — subscribe_alarms.ts**

```typescript
const host = process.argv[2] || "http://localhost:3000";
const apiKey = process.argv[3] || "";
const url = `${host}/api/v1/stream`;

console.log(`Connecting to ${url}...`);

const headers: Record<string, string> = {};
if (apiKey) headers["X-API-Key"] = apiKey;

fetch(url, { headers })
  .then((res) => {
    const reader = res.body?.getReader();
    const decoder = new TextDecoder();

    function read(): Promise<void> {
      return reader!.read().then(({ done, value }) => {
        if (done) return;
        const text = decoder.decode(value);
        for (const line of text.split("\n")) {
          if (line.startsWith("data:")) {
            try {
              const event = JSON.parse(line.slice(5));
              if (event.type === "alarm") {
                const alarm = event.data;
                console.log(`[ALARM] ${(alarm.severity || "?").toUpperCase()}: ${alarm.message || "N/A"}`);
              }
            } catch {}
          }
        }
        return read();
      });
    }
    return read();
  })
  .catch(console.error);
```

- [ ] **Step 7: Commit**

```bash
git add sdks/python/examples/ sdks/typescript/examples/
git commit -m "feat(sdk): add Python and TypeScript example scripts (read, write, subscribe)"
```

---

### Task 11: Update Docs, Landing Page, and Changelog

**Files:**
- Modify: `voltrus-landing/docs.html` (add API reference + SDK quickstart section)
- Modify: `voltrus-landing/index.html` (add Developer SDKs feature)
- Modify: `voltrus-landing/changelog.html` (add v0.27.0 entry)
- Modify: `Cargo.toml` (bump version from 0.25.0 to 0.27.0)

- [ ] **Step 1: Bump version in `Cargo.toml`**

Change `version = "0.25.0"` to `version = "0.27.0"` in `[package]`.

- [ ] **Step 2: Add v0.27.0 changelog entry in `voltrus-landing/changelog.html`**

Add a new entry at the top of the changelog list:
```html
<div class="changelog-entry">
    <h3>v0.27.0 — SDKs & OpenAPI</h3>
    <p>OpenAPI 3.1 spec with interactive Swagger UI, Python SDK (PyPI), TypeScript SDK (npm), example scripts.</p>
    <ul>
        <li>OpenAPI 3.1 spec auto-generated from all /api/v1/* endpoints via utoipa</li>
        <li>Interactive Swagger UI at /api/docs for live API exploration</li>
        <li>Python SDK with pip install, API key auth, data/alarm/command examples</li>
        <li>TypeScript SDK with npm install, Axios-based, full type safety</li>
        <li><code>voltrus --dump-openapi</code> CLI for spec export</li>
        <li>SDK example scripts: read tags, write setpoints, subscribe to alarms</li>
    </ul>
</div>
```

- [ ] **Step 3: Add Developer SDKs feature to landing page in `voltrus-landing/index.html`**

Find the features section and add:
```html
<div class="feature-card">
    <h3>Developer SDKs</h3>
    <p>Official Python and TypeScript SDKs auto-generated from OpenAPI spec. Interactive Swagger UI for API exploration. Read tags, write setpoints, subscribe to alarms — in 5 lines of code.</p>
</div>
```

- [ ] **Step 4: Add SDK quickstart to docs in `voltrus-landing/docs.html`**

Add a new section:
```html
<section id="api-reference">
    <h2>API Reference & SDKs</h2>
    <h3>Interactive API Explorer</h3>
    <p>Navigate to <code>/api/docs</code> on your Voltrus instance for the interactive Swagger UI. All endpoints are documented with request/response schemas and "Try it out" functionality.</p>

    <h3>OpenAPI Spec</h3>
    <p>The raw OpenAPI 3.1 spec is available at <code>/api/openapi.json</code>. Use it with any OpenAPI-compatible tool (Postman, Insomnia, code generators).</p>
    <pre><code>voltrus --dump-openapi > openapi.json</code></pre>

    <h3>Python SDK</h3>
    <pre><code>pip install voltrus</code></pre>
    <pre><code>from voltrus import ApiClient, Configuration
from voltrus.api.data_api import DataApi

config = Configuration(host="http://192.168.1.100:3000")
config.api_key["api_key"] = "vt_sk_abc123"

with ApiClient(config) as client:
    data = DataApi(client).get_data(screen="overview")
    for ds in data:
        print(f"{ds.key}: {ds.value}")</code></pre>

    <h3>TypeScript SDK</h3>
    <pre><code>npm install voltrus</code></pre>
    <pre><code>import { Configuration, DataApi } from "voltrus";

const config = new Configuration({
  basePath: "http://192.168.1.100:3000",
  apiKey: "vt_sk_abc123",
});

const data = await new DataApi(config).getData({ screen: "overview" });
data.forEach(ds => console.log(`${ds.key}: ${ds.value}`));</code></pre>
</section>
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock voltrus-landing/
git commit -m "feat: v0.27.0 — bump version, update docs, landing page, and changelog"
```

---

### Task 12: Final Integration Test + Version Commit

**Files:**
- No new files — validation only

- [ ] **Step 1: Run full test suite**

Run: `cargo test 2>&1 | tail -5`
Expected: All tests pass

- [ ] **Step 2: Build release binary**

Run: `cargo build --release 2>&1 | tail -3`
Expected: Build succeeds

- [ ] **Step 3: Verify --dump-openapi outputs valid spec**

Run: `./target/release/voltrus-lite-rust-service --dump-openapi | python3 -m json.tool | head -20`
Expected: Valid JSON with `openapi`, `info`, `paths`, `components`

- [ ] **Step 4: Verify binary size within budget**

Run: `ls -lh target/release/voltrus-lite-rust-service | awk '{print $5}'`
Expected: <30MB (the spec's performance budget)

- [ ] **Step 5: Final commit (tag release)**

```bash
git tag v0.27.0
git log --oneline -15
```

Review the commit history to verify all tasks are present.
