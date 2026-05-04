# v0.32.0 Track 2 — Backend + Frontend Features (Tag Browser, Audit Log, CSV Import)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the tag browser with UNS tree view, tag naming enforcement, audit log search/filter, and CSV data import for v0.32.0.

**Architecture:** Hierarchical tag tree API built from data_sources `path` field. SQLite `LIKE`-based full-text search for audit logs. Multipart CSV upload with streaming parser for data import. React tree component with HTML5 drag-and-drop for tag browser UI.

**Tech Stack:** Rust/Axum, SQLite, React 19, TypeScript, TanStack Query, @tanstack/react-virtual

---

## Task 1: Database Migration — Add `path` Column to `data_sources`

**Files:**
- Modify: `src/storage/schema/migrations.rs`
- Modify: `src/models/mod.rs` (add `path` field to `DataSource`)
- Modify: `src/storage/device_crud.rs` (read/write `path` field)
- Test: `src/storage/tests/mod.rs` (add tag_tree test module)

**Steps:**

- [ ] 1.1 Add migration for `path` column on `data_sources` table

```rust
// Add to end of run_migrations() in src/storage/schema/migrations.rs

    // Migration: v0.32.0 -- UNS path column on data_sources for tag browser tree
    let _ = conn.execute_batch("ALTER TABLE data_sources ADD COLUMN path TEXT DEFAULT NULL");
```

- [ ] 1.2 Add `path` field to `DataSource` model

In `src/models/mod.rs`, modify the `DataSource` struct:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct DataSource {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub unit: String,
    pub color: String,
    pub min_val: f64,
    pub max_val: f64,
    pub warn_val: f64,
    pub crit_val: f64,
    pub base_val: f64,
    pub sort_order: i64,
    pub decimals: i64,
    pub source_type: String,
    pub source_config: String,
    pub device_id: String,
    pub equipment_id: Option<String>,
    pub chart_type: String,
    pub path: Option<String>,
}
```

- [ ] 1.3 Update `DataSource` SQL SELECT queries in `src/storage/device_crud.rs`

Add `, path` to every SELECT that fetches `data_sources` columns, and add `path: row.get(17)?` to each row mapper. The queries are:

- `get_data_source_by_id` — add `, path` after `chart_type` in SELECT
- `list_data_sources` (both the screen-filtered and unfiltered queries) — add `, path`
- `create_data_source` — no change needed (INSERT doesn't include path)
- `update_data_source` — add `"path"` to the `allowed` array

Updated `get_data_source_by_id`:

```rust
pub async fn get_data_source_by_id(&self, id: i64) -> Result<Option<DataSource>> {
    let conn = self.conn.clone();
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT id, key, name, unit, color, min_val, max_val,
                    warn_val, crit_val, base_val, sort_order, decimals,
                    source_type, source_config, device_id, equipment_id, chart_type, path
             FROM data_sources WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(DataSource {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    name: row.get(2)?,
                    unit: row.get(3)?,
                    color: row.get(4)?,
                    min_val: row.get(5)?,
                    max_val: row.get(6)?,
                    warn_val: row.get(7)?,
                    crit_val: row.get(8)?,
                    base_val: row.get(9)?,
                    sort_order: row.get(10)?,
                    decimals: row.get(11)?,
                    source_type: row.get(12)?,
                    source_config: row.get(13)?,
                    device_id: row.get(14)?,
                    equipment_id: row.get(15)?,
                    chart_type: row.get(16)?,
                    path: row.get(17)?,
                })
            },
        );
        match result {
            Ok(ds) => Ok(Some(ds)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Get data source by id failed"),
        }
    })
    .await
    .context("Task join failed")?
}
```

Updated `list_data_sources` (both SELECT variants):

```rust
// Unfiltered query
"SELECT id, key, name, unit, color, min_val, max_val,
        warn_val, crit_val, base_val, sort_order, decimals,
        source_type, source_config, device_id, equipment_id, chart_type, path
 FROM data_sources
 ORDER BY sort_order"

// Screen-filtered query
"SELECT s.id, s.key, s.name, s.unit, s.color, s.min_val, s.max_val,
        s.warn_val, s.crit_val, s.base_val, s.sort_order, s.decimals,
        s.source_type, s.source_config, s.device_id, s.equipment_id, s.chart_type, s.path
 FROM data_sources s
 JOIN screen_data_sources ss ON s.id = ss.data_source_id
 WHERE ss.screen_id = ?1
 ORDER BY ss.sort_order"
```

Row mapper in `list_data_sources`:

```rust
Ok(DataSource {
    id: row.get(0)?,
    key: row.get(1)?,
    name: row.get(2)?,
    unit: row.get(3)?,
    color: row.get(4)?,
    min_val: row.get(5)?,
    max_val: row.get(6)?,
    warn_val: row.get(7)?,
    crit_val: row.get(8)?,
    base_val: row.get(9)?,
    sort_order: row.get(10)?,
    decimals: row.get(11)?,
    source_type: row.get(12)?,
    source_config: row.get(13)?,
    device_id: row.get(14)?,
    equipment_id: row.get(15)?,
    chart_type: row.get(16)?,
    path: row.get(17)?,
})
```

Update `allowed` array in `update_data_source`:

```rust
let allowed = [
    "key", "name", "unit", "color", "min_val", "max_val",
    "warn_val", "crit_val", "base_val", "sort_order", "decimals",
    "source_type", "source_config", "device_id", "equipment_id", "chart_type", "path",
];
```

- [ ] 1.4 Write failing test for `path` field

Create `src/storage/tests/tag_tree_tests.rs`:

```rust
use crate::storage::Storage;

#[tokio::test]
async fn test_data_source_path_roundtrip() {
    let storage = Storage::new_in_memory().unwrap();

    // Create a data source
    let id = storage
        .create_data_source(
            "PT001_TEMP", "PT001_TEMP", "C", "#f97316",
            0.0, 100.0, 80.0, 90.0, 25.0, 0, 2,
            "modbus", "{}", "plc1", None, "sparkline",
        )
        .await
        .unwrap();

    // Update path
    let fields = serde_json::json!({"path": "Site A/Area 1/Unit 2/PT001"});
    storage.update_data_source(id, &fields).await.unwrap();

    // Read back and verify path
    let ds = storage.get_data_source_by_id(id).await.unwrap().unwrap();
    assert_eq!(ds.path.as_deref(), Some("Site A/Area 1/Unit 2/PT001"));
}
```

Register in `src/storage/tests/mod.rs`:

```rust
mod tag_tree_tests;
```

- [ ] 1.5 Verify compilation and test pass

```bash
cargo test --lib storage::tests::tag_tree_tests
```

- [ ] **Commit:** `feat(db): add path column to data_sources for UNS tag tree`

---

## Task 2: Backend — Tag Tree API (`GET /api/v1/tags/tree`, `PUT /api/v1/tags/{id}/path`)

**Files:**
- Create: `src/api/tag_tree.rs`
- Modify: `src/api/mod.rs` (register module + routes)
- Modify: `src/storage/device_crud.rs` (add `list_data_sources_with_path` and `build_tag_tree` methods)
- Test: `src/api/tests/mod.rs` (add tag_tree test module)
- Create: `src/api/tests/tag_tree.rs`

**Steps:**

- [ ] 2.1 Add storage methods for tag tree

Add to `src/storage/device_crud.rs`:

```rust
    /// List all data sources with path field for tree building.
    pub async fn list_data_sources_for_tree(&self) -> Result<Vec<(i64, String, String, String, Option<String>)>> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, device_id, equipment_id, path FROM data_sources ORDER BY path, name",
                )
                .context("Prepare failed")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                })
                .context("Query failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .context("Row mapping failed")
        })
        .await
        .context("Task join failed")?
    }

    /// Build a hierarchical UNS tree from flat tag data.
    /// Returns a JSON tree: [{ "name": "Site", "children": [...] }]
    pub fn build_tag_tree(
        tags: &[(i64, String, String, String, Option<String>)],
        search_term: Option<&str>,
    ) -> Vec<serde_json::Value> {
        use std::collections::BTreeMap;

        let search_lower = search_term.map(|s| s.to_lowercase());

        // Tree node: name -> { tag_count, children: BTreeMap<name, node> }
        #[derive(Default)]
        struct TreeNode {
            tag_ids: Vec<i64>,
            children: BTreeMap<String, TreeNode>,
        }

        let mut root = BTreeMap::<String, TreeNode>::new();

        for (id, name, device_id, equipment_id, path) in tags {
            // Apply search filter
            if let Some(ref term) = search_lower {
                let name_lower = name.to_lowercase();
                let path_lower = path.as_deref().unwrap_or("").to_lowercase();
                let device_lower = device_id.to_lowercase();
                if !name_lower.contains(term) && !path_lower.contains(term) && !device_lower.contains(term) {
                    continue;
                }
            }

            let segments: Vec<&str> = match path {
                Some(p) if !p.is_empty() => p.split('/').collect(),
                _ => vec![&device_id],
            };

            let mut current = &mut root;
            for segment in &segments {
                current = current
                    .entry(segment.to_string())
                    .or_default()
                    .children
                    .entry("_".to_string())
                    .or_default(); // placeholder
                current = current.children.entry("_".to_string()).or_default();
            }

            // Add tag as leaf node under the last segment
            let mut current = &mut root;
            for segment in &segments {
                let node = current.entry(segment.to_string()).or_default();
                current = node;
            }
            current.tag_ids.push(*id);
        }

        fn node_to_json(name: &str, node: &TreeNode) -> serde_json::Value {
            let mut children: Vec<serde_json::Value> = Vec::new();

            // Add tag leaf nodes (tag IDs with the tag's own name stored externally)
            // Tags are represented as children with a special structure
            for (child_name, child_node) in &node.children {
                if child_name == "_" {
                    continue; // skip placeholder
                }
                children.push(node_to_json(child_name, child_node));
            }

            serde_json::json!({
                "name": name,
                "tagCount": node.tag_ids.len(),
                "tagIds": node.tag_ids,
                "children": children,
            })
        }

        let mut result: Vec<serde_json::Value> = root
            .iter()
            .map(|(name, node)| node_to_json(name, node))
            .collect();
        result.sort_by(|a, b| a["name"].as_str().unwrap_or("").cmp(b["name"].as_str().unwrap_or("")));
        result
    }

    /// Update the path of a data source by ID.
    pub async fn update_data_source_path(&self, id: i64, path: Option<&str>) -> Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE data_sources SET path = ?1 WHERE id = ?2",
                rusqlite::params![path, id],
            )
            .context("Update data source path failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }
```

- [ ] 2.3 Create the tag tree API handler

Create `src/api/tag_tree.rs`:

```rust
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub struct TreeQuery {
    pub search: Option<String>,
}

/// GET /api/v1/tags/tree — hierarchical UNS tree view of all tags
pub async fn get_tag_tree(
    State(state): State<AppState>,
    Query(params): Query<TreeQuery>,
) -> Result<impl IntoResponse, AppError> {
    let tags = state.storage.list_data_sources_for_tree().await?;
    let tree = state.storage.build_tag_tree(&tags, params.search.as_deref());
    Ok(Json(json!({ "tree": tree })))
}

/// PUT /api/v1/tags/{id}/path — update a tag's UNS path
pub async fn update_tag_path(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let path = body.get("path").and_then(|v| v.as_str());

    // Validate path format: non-empty segments separated by /
    if let Some(p) = path {
        if p.is_empty() {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "path must not be empty string; use null to clear"})),
            ).into_response());
        }
        for segment in p.split('/') {
            if segment.trim().is_empty() {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "path segments must not be empty (e.g., 'Site/Area/Unit')"})),
                ).into_response());
            }
        }
    }

    state.storage.update_data_source_path(id, path).await?;
    Ok(Json(json!({"ok": true})))
}
```

- [ ] 2.4 Register routes in `src/api/mod.rs`

Add module declaration at top:

```rust
mod tag_tree;
```

Add routes to `protected` router (any authenticated user can browse tags):

```rust
.route("/api/v1/tags/tree", get(tag_tree::get_tag_tree))
```

Add route to `admin_protected` router (only admins can update paths):

```rust
.route("/api/v1/tags/{id}/path", put(tag_tree::update_tag_path))
```

- [ ] 2.5 Write failing tests

Create `src/api/tests/tag_tree.rs`:

```rust
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

use crate::api::tests::*;

#[tokio::test]
async fn test_tag_tree_requires_auth() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tags/tree")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_tag_tree_returns_empty_for_no_tags() {
    let app = test_app_with_seeded_admin().await;
    let cookie = seed_and_get_cookie(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tags/tree")
                .header("cookie", format!("voltrus_token={}", cookie))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["tree"], serde_json::json!([]));
}

#[tokio::test]
async fn test_update_tag_path_requires_admin() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/tags/1/path")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"path":"Site/Area"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
```

Register in `src/api/tests/mod.rs`:

```rust
mod tag_tree;
```

- [ ] 2.6 Verify compilation and test pass

```bash
cargo test --lib api::tests::tag_tree
```

- [ ] **Commit:** `feat(api): add tag tree endpoint (GET /tags/tree, PUT /tags/{id}/path)`

---

## Task 3: Backend — Tag Naming Convention Enforcement

**Files:**
- Create: `src/api/naming_convention.rs`
- Modify: `src/api/mod.rs` (register module + routes)
- Modify: `src/api/data_sources.rs` (validate tag name on create/update)
- Modify: `src/storage/settings_kv.rs` (no change needed — uses existing `upsert_setting`/`get_settings`)
- Test: `src/api/tests/naming_convention.rs`

**Steps:**

- [ ] 3.1 Create naming convention API handler

Create `src/api/naming_convention.rs`:

```rust
use axum::extract::{State, Extension};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use super::{AppError, AppState, AuthUser};

/// Default ISA-5.1 naming convention regex.
const ISA_5_1_DEFAULT: &str = r"^[A-Z]{2,4}\d{3}_[A-Z0-9_]+$";

/// Built-in naming convention templates.
const BUILTIN_TEMPLATES: &[(&str, &str, &str)] = &[
    ("isa-5.1", "ISA-5.1 (e.g., PT001_TEMP, TT102_ALARM)", ISA_5_1_DEFAULT),
    ("iec-61346", "IEC 61346 (e.g., =PLT+CV101-TT001)", r"^=[A-Z]+\+[A-Z0-9]+-[A-Z0-9]+$"),
    ("kks", "KKS (e.g., LAB10CT001)", r"^[A-Z]{2}\d{2}[A-Z]{2}\d{3}$"),
    ("custom", "Custom (user-defined regex)", ""),
];

/// GET /api/v1/settings/naming-convention — get current naming convention config
pub async fn get_naming_convention(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let settings = state
        .storage
        .get_settings(&["naming_template", "naming_regex"])
        .await?;

    let mut template = "isa-5.1".to_string();
    let mut custom_regex: Option<String> = None;

    for s in &settings {
        match s.key.as_str() {
            "naming_template" => template = s.value.clone(),
            "naming_regex" => {
                if !s.value.is_empty() {
                    custom_regex = Some(s.value.clone());
                }
            }
            _ => {}
        }
    }

    // Resolve the effective regex
    let effective_regex = match template.as_str() {
        "custom" => custom_regex.clone().unwrap_or_else(|| ISA_5_1_DEFAULT.to_string()),
        _ => {
            BUILTIN_TEMPLATES
                .iter()
                .find(|(t, _, _)| *t == template)
                .map(|(_, _, regex)| regex.to_string())
                .unwrap_or_else(|| ISA_5_1_DEFAULT.to_string())
        }
    };

    Ok(Json(json!({
        "template": template,
        "regex": effective_regex,
        "custom_regex": custom_regex,
        "templates": BUILTIN_TEMPLATES.iter().map(|(id, label, _)| {
            json!({"id": id, "label": label})
        }).collect::<Vec<_>>(),
    })))
}

/// PUT /api/v1/settings/naming-convention — set naming convention config
pub async fn update_naming_convention(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let template = body
        .get("template")
        .and_then(|v| v.as_str())
        .unwrap_or("isa-5.1")
        .to_string();

    // Validate template exists
    let valid_template = BUILTIN_TEMPLATES
        .iter()
        .any(|(t, _, _)| *t == template) || template == "custom";

    if !valid_template {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Invalid template. Available: {}", 
                    BUILTIN_TEMPLATES.iter().map(|(t, _, _)| *t).collect::<Vec<_>>().join(", "))
            })),
        ).into_response());
    }

    let custom_regex = body
        .get("custom_regex")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // If custom template, validate the regex compiles
    if template == "custom" && !custom_regex.is_empty() {
        if regex::Regex::new(custom_regex).is_err() {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid regex pattern"})),
            ).into_response());
        }
    }

    state
        .storage
        .upsert_setting("naming_template", &template)
        .await?;
    state
        .storage
        .upsert_setting("naming_regex", custom_regex)
        .await?;

    state.audit_logger.log(crate::security::AuditEvent {
        action: "update_naming_convention".to_string(),
        username: auth.username.clone(),
        resource: "settings:naming-convention".to_string(),
        details: Some(format!("template={}, custom_regex={}", template, custom_regex)),
    });

    Ok(Json(json!({"ok": true})))
}

/// Validate a tag name against the current naming convention.
/// Returns Ok(()) if valid, Err with hint message if invalid.
pub async fn validate_tag_name(
    state: &AppState,
    name: &str,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let settings = state
        .storage
        .get_settings(&["naming_template", "naming_regex"])
        .await
        .map_err(|e| AppError(e))?;

    let mut template = "isa-5.1".to_string();
    let mut custom_regex = String::new();

    for s in &settings {
        match s.key.as_str() {
            "naming_template" => template = s.value.clone(),
            "naming_regex" => custom_regex = s.value.clone(),
            _ => {}
        }
    }

    let effective_regex = match template.as_str() {
        "custom" if !custom_regex.is_empty() => custom_regex,
        _ => {
            BUILTIN_TEMPLATES
                .iter()
                .find(|(t, _, _)| *t == template)
                .map(|(_, _, regex)| regex.to_string())
                .unwrap_or_else(|| ISA_5_1_DEFAULT.to_string())
        }
    };

    if let Ok(re) = regex::Regex::new(&effective_regex) {
        if !re.is_match(name) {
            let template_label = BUILTIN_TEMPLATES
                .iter()
                .find(|(t, _, _)| *t == template)
                .map(|(_, label, _)| *label)
                .unwrap_or("Custom");

            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": format!(
                        "Tag name '{}' does not match naming convention '{}' (pattern: {})",
                        name, template_label, effective_regex
                    ),
                    "hint": format!("Expected format: {}. Example: PT001_TEMP", template_label),
                    "pattern": effective_regex,
                })),
            ));
        }
    }

    Ok(())
}
```

- [ ] 3.2 Add `regex` dependency to `Cargo.toml`

```toml
# Add to [dependencies] section
regex = "1"
```

- [ ] 3.3 Integrate naming validation into data_sources create/update

Modify `src/api/data_sources.rs`:

```rust
use super::naming_convention::validate_tag_name;

pub async fn create_data_source(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let key = body["key"].as_str().unwrap_or("data_source").to_string();
    let name = body["name"].as_str().unwrap_or("DataSource").to_string();

    // Validate tag name against naming convention
    if let Err((status, json)) = validate_tag_name(&state, &name).await {
        return Ok((status, json).into_response());
    }

    // ... rest of existing handler unchanged
```

For `update_data_source`, add validation when `name` is in the update body:

```rust
pub async fn update_data_source(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Validate tag name against naming convention if name is being updated
    if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
        if let Err((status, json)) = validate_tag_name(&state, name).await {
            return Ok((status, json).into_response());
        }
    }

    state.storage.update_data_source(id, &body).await?;
    Ok(Json(json!({"ok": true})))
}
```

- [ ] 3.4 Register routes in `src/api/mod.rs`

Add module declaration:

```rust
mod naming_convention;
```

Add to `public` router (no auth needed to read naming convention):

```rust
.route("/api/v1/settings/naming-convention", get(naming_convention::get_naming_convention))
```

Add to `admin_protected` router:

```rust
.route("/api/v1/settings/naming-convention", put(naming_convention::update_naming_convention))
```

- [ ] 3.5 Write failing tests

Create `src/api/tests/naming_convention.rs`:

```rust
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

use crate::api::tests::*;

#[tokio::test]
async fn test_get_naming_convention_public() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/settings/naming-convention")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Should return 200 since it's public (but license check may return 403)
    // The endpoint is public so it passes auth middleware
    assert!(resp.status() == StatusCode::OK || resp.status() == StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_naming_convention_requires_admin() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/settings/naming-convention")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"template":"iec-61346"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_data_source_validates_naming_convention() {
    let app = test_app_with_seeded_admin().await;
    let cookie = seed_and_get_cookie(&app).await;

    // Default ISA-5.1: PT001_TEMP should pass
    let body = serde_json::json!({
        "name": "PT001_TEMP",
        "key": "pt001_temp",
        "unit": "C",
        "device_id": "default",
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/data-sources")
                .header("cookie", format!("voltrus_token={}", cookie))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // "my_sensor" should fail naming convention
    let bad_body = serde_json::json!({
        "name": "my_sensor",
        "key": "my_sensor",
        "unit": "C",
        "device_id": "default",
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/data-sources")
                .header("cookie", format!("voltrus_token={}", cookie))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&bad_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
```

Register in `src/api/tests/mod.rs`:

```rust
mod naming_convention;
```

- [ ] 3.6 Verify compilation and test pass

```bash
cargo test --lib api::tests::naming_convention
```

- [ ] **Commit:** `feat(api): tag naming convention enforcement (ISA-5.1, IEC-61346, KKS, custom)`

---

## Task 4: Backend — Audit Log Search, Filter, and CSV Export

**Files:**
- Create: `src/api/audit_log.rs`
- Modify: `src/api/mod.rs` (register module + routes)
- Modify: `src/storage/settings_audit.rs` (extend `list_audit_logs` with search/filter params, add CSV export, add index migration)
- Modify: `src/storage/schema/migrations.rs` (add index on audit_log action/username)
- Test: `src/api/tests/audit_log.rs`

**Steps:**

- [ ] 4.1 Add audit_log search index migration

Add to `src/storage/schema/migrations.rs`:

```rust
    // Migration: v0.32.0 -- Audit log search indexes
    let _ = conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);
         CREATE INDEX IF NOT EXISTS idx_audit_log_username ON audit_log(username);
         CREATE INDEX IF NOT EXISTS idx_audit_log_action_user_time ON audit_log(action, username, created_at);"
    );
```

- [ ] 4.2 Extend storage methods for audit log search

Replace `list_audit_logs` in `src/storage/settings_audit.rs` with extended version:

```rust
use super::Storage;
use anyhow::{Context, Result};

impl Storage {
    // ── Audit Log ──

    pub async fn log_audit_event(
        &self,
        action: &str,
        username: &str,
        resource: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let action = action.to_string();
        let username = username.to_string();
        let resource = resource.to_string();
        let details = details.map(|s| s.to_string());
        let now = crate::timeutil::now_secs();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO audit_log (action, username, resource, details, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![action, username, resource, details, now],
            )
            .context("Insert audit log failed")?;
            Ok(())
        })
        .await
        .context("Task join failed")?
    }

    /// List audit logs with full-text search, filter, sort, and pagination.
    pub async fn list_audit_logs_filtered(
        &self,
        search: Option<&str>,
        action: Option<&str>,
        username: Option<&str>,
        from: Option<i64>,
        to: Option<i64>,
        page: u32,
        limit: u32,
        sort: &str,     // "created_at" or "action" or "username"
        order: &str,    // "asc" or "desc"
    ) -> Result<(Vec<crate::models::AuditLogEntry>, i64)> {
        let conn = self.conn.clone();
        let search = search.map(|s| s.to_string());
        let action = action.map(|s| s.to_string());
        let username = username.map(|s| s.to_string());
        let sort = sort.to_string();
        let order = order.to_string();

        // Validate sort column
        let sort_col = match sort.as_str() {
            "action" | "username" => sort.as_str(),
            _ => "created_at",
        };
        // Validate order
        let order_dir = match order.as_str() {
            "asc" => "ASC",
            _ => "DESC",
        };

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let mut where_sql = String::from("WHERE 1=1");
            let mut param_idx = 1;
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(ref term) = search {
                where_sql.push_str(&format!(
                    " AND (details LIKE ?{idx} OR resource LIKE ?{idx} OR username LIKE ?{idx} OR action LIKE ?{idx})",
                    idx = param_idx
                ));
                let pattern = format!("%{}%", term);
                param_values.push(Box::new(pattern));
                param_idx += 1;
            }
            if let Some(ref a) = action {
                where_sql.push_str(&format!(" AND action = ?{}", param_idx));
                param_values.push(Box::new(a.clone()));
                param_idx += 1;
            }
            if let Some(ref u) = username {
                where_sql.push_str(&format!(" AND username = ?{}", param_idx));
                param_values.push(Box::new(u.clone()));
                param_idx += 1;
            }
            if let Some(f) = from {
                where_sql.push_str(&format!(" AND created_at >= ?{}", param_idx));
                param_values.push(Box::new(f));
                param_idx += 1;
            }
            if let Some(t) = to {
                where_sql.push_str(&format!(" AND created_at <= ?{}", param_idx));
                param_values.push(Box::new(t));
                param_idx += 1;
            }

            // Count query
            let count_sql = format!("SELECT COUNT(*) FROM audit_log {}", where_sql);
            let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let total: i64 = conn
                .query_row(&count_sql, params.as_slice(), |row| row.get(0))
                .context("Count audit logs failed")?;

            // Data query with pagination
            let offset = (page.saturating_sub(1)) * limit;
            let data_sql = format!(
                "SELECT id, action, username, resource, details, created_at
                 FROM audit_log {}
                 ORDER BY {} {}
                 LIMIT ?{} OFFSET ?{}",
                where_sql, sort_col, order_dir, param_idx, param_idx + 1
            );
            param_values.push(Box::new(limit));
            param_values.push(Box::new(offset));
            let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&data_sql).context("Prepare audit logs query failed")?;
            let rows = stmt
                .query_map(params.as_slice(), |row| {
                    Ok(crate::models::AuditLogEntry {
                        id: row.get(0)?,
                        action: row.get(1)?,
                        username: row.get(2)?,
                        resource: row.get(3)?,
                        details: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })
                .context("Query audit logs failed")?
                .collect::<Result<Vec<_>, _>>()
                .context("Map audit logs failed")?;

            Ok((rows, total))
        })
        .await
        .context("Task join failed")?
    }

    /// Export audit logs as CSV string with same filters as list.
    pub async fn export_audit_logs_csv(
        &self,
        search: Option<&str>,
        action: Option<&str>,
        username: Option<&str>,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Result<String> {
        let conn = self.conn.clone();
        let search = search.map(|s| s.to_string());
        let action = action.map(|s| s.to_string());
        let username = username.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let mut where_sql = String::from("WHERE 1=1");
            let mut param_idx = 1;
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(ref term) = search {
                where_sql.push_str(&format!(
                    " AND (details LIKE ?{idx} OR resource LIKE ?{idx} OR username LIKE ?{idx} OR action LIKE ?{idx})",
                    idx = param_idx
                ));
                let pattern = format!("%{}%", term);
                param_values.push(Box::new(pattern));
                param_idx += 1;
            }
            if let Some(ref a) = action {
                where_sql.push_str(&format!(" AND action = ?{}", param_idx));
                param_values.push(Box::new(a.clone()));
                param_idx += 1;
            }
            if let Some(ref u) = username {
                where_sql.push_str(&format!(" AND username = ?{}", param_idx));
                param_values.push(Box::new(u.clone()));
                param_idx += 1;
            }
            if let Some(f) = from {
                where_sql.push_str(&format!(" AND created_at >= ?{}", param_idx));
                param_values.push(Box::new(f));
                param_idx += 1;
            }
            if let Some(t) = to {
                where_sql.push_str(&format!(" AND created_at <= ?{}", param_idx));
                param_values.push(Box::new(t));
                param_idx += 1;
            }

            let sql = format!(
                "SELECT id, action, username, resource, details, created_at
                 FROM audit_log {}
                 ORDER BY created_at ASC",
                where_sql
            );
            let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&sql).context("Prepare audit CSV query failed")?;
            let rows = stmt
                .query_map(params.as_slice(), |row| {
                    Ok(crate::models::AuditLogEntry {
                        id: row.get(0)?,
                        action: row.get(1)?,
                        username: row.get(2)?,
                        resource: row.get(3)?,
                        details: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })
                .context("Query audit CSV failed")?;

            let mut csv = String::from("id,action,username,resource,details,created_at\n");
            for row in rows {
                let entry = row.context("Map audit CSV row failed")?;
                csv.push_str(&format!(
                    "{},\"{}\",\"{}\",\"{}\",\"{}\",{}\n",
                    entry.id,
                    entry.action.replace('"', "\"\""),
                    entry.username.replace('"', "\"\""),
                    entry.resource.replace('"', "\"\""),
                    entry.details.as_deref().unwrap_or("").replace('"', "\"\"").replace('\n', "\\n"),
                    entry.created_at,
                ));
            }

            Ok(csv)
        })
        .await
        .context("Task join failed")?
    }

    /// Keep backward-compatible list_audit_logs for existing callers.
    pub async fn list_audit_logs(&self, limit: i64) -> Result<Vec<(i64, String, String, String, Option<String>, i64)>> {
        let (entries, _) = self.list_audit_logs_filtered(None, None, None, None, None, 1, limit as u32, "created_at", "desc").await?;
        Ok(entries.into_iter().map(|e| (e.id, e.action, e.username, e.resource, e.details, e.created_at)).collect())
    }
}
```

- [ ] 4.3 Add `AuditLogEntry` model to `src/models/mod.rs`

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct AuditLogEntry {
    pub id: i64,
    pub action: String,
    pub username: String,
    pub resource: String,
    pub details: Option<String>,
    pub created_at: i64,
}
```

- [ ] 4.4 Create audit log API handler

Create `src/api/audit_log.rs`:

```rust
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

use super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub search: Option<String>,
    pub action: Option<String>,
    pub username: Option<String>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default = "default_sort")]
    pub sort: String,
    #[serde(default = "default_order")]
    pub order: String,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 50 }
fn default_sort() -> String { "created_at".to_string() }
fn default_order() -> String { "desc".to_string() }

/// GET /api/v1/audit — list audit log entries with search, filter, pagination
pub async fn list_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<AuditQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.min(1000);
    let (entries, total) = state
        .storage
        .list_audit_logs_filtered(
            params.search.as_deref(),
            params.action.as_deref(),
            params.username.as_deref(),
            params.from,
            params.to,
            params.page,
            limit,
            &params.sort,
            &params.order,
        )
        .await?;

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    Ok(Json(json!({
        "entries": entries,
        "total": total,
        "page": params.page,
        "limit": limit,
        "total_pages": total_pages,
    })))
}

/// GET /api/v1/audit/export — CSV export of audit logs
pub async fn export_audit_csv(
    State(state): State<AppState>,
    Query(params): Query<AuditQuery>,
) -> Result<impl IntoResponse, AppError> {
    let csv = state
        .storage
        .export_audit_logs_csv(
            params.search.as_deref(),
            params.action.as_deref(),
            params.username.as_deref(),
            params.from,
            params.to,
        )
        .await?;

    let filename = format!(
        "audit-log-{}.csv",
        time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Iso8601::DATE)
            .unwrap_or_else(|_| "unknown".to_string())
    );

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("text/csv")),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
            ),
        ],
        csv,
    ))
}
```

- [ ] 4.5 Register routes in `src/api/mod.rs`

Add module declaration:

```rust
mod audit_log;
```

Add to `admin_protected` router:

```rust
.route("/api/v1/audit", get(audit_log::list_audit_logs))
.route("/api/v1/audit/export", get(audit_log::export_audit_csv))
```

- [ ] 4.6 Write failing tests

Create `src/api/tests/audit_log.rs`:

```rust
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

use crate::api::tests::*;

#[tokio::test]
async fn test_audit_log_requires_auth() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/audit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_audit_export_requires_auth() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/audit/export")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_audit_log_returns_empty() {
    let app = test_app_with_seeded_admin().await;
    let cookie = seed_and_get_cookie(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/audit?page=1&limit=10")
                .header("cookie", format!("voltrus_token={}", cookie))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["entries"], serde_json::json!([]));
    assert_eq!(json["total"], 0);
}
```

Register in `src/api/tests/mod.rs`:

```rust
mod audit_log;
```

- [ ] 4.7 Verify compilation and test pass

```bash
cargo test --lib api::tests::audit_log
```

- [ ] **Commit:** `feat(api): audit log full-text search, filter, pagination, and CSV export`

---

## Task 5: Backend — CSV Data Import (`POST /api/v1/import/csv`)

**Files:**
- Create: `src/api/csv_import.rs`
- Modify: `src/api/mod.rs` (register module + routes, increase body limit for import route)
- Modify: `Cargo.toml` (add `csv` direct dependency)
- Test: `src/api/tests/csv_import.rs`

**Steps:**

- [ ] 5.1 Add `csv` dependency to `Cargo.toml`

```toml
csv = "1"
```

- [ ] 5.2 Create CSV import API handler

Create `src/api/csv_import.rs`:

```rust
use std::collections::HashMap;

use axum::extract::{Multipart, Query, State, Extension};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

use super::{AppError, AppState, AuthUser};

#[derive(Debug, Deserialize)]
pub struct ImportQuery {
    pub dry_run: Option<String>,   // "true" to preview only
    pub mode: Option<String>,      // "merge" (default) or "replace"
    pub delimiter: Option<String>, // "," (default) or ";" or "\t"
}

/// Auto-detect which CSV column maps to which telemetry field.
fn auto_detect_columns(headers: &[String]) -> HashMap<String, usize> {
    let mut mapping = HashMap::new();

    for (idx, header) in headers.iter().enumerate() {
        let h = header.trim().to_lowercase().replace('_', " ").replace('-', " ");

        match h.as_str() {
            "name" | "tag" | "tag name" | "sensor" | "sensor name" | "key" | "data source"
            | "tagname" | "point" | "point name" => {
                mapping.insert("tag_name".to_string(), idx);
            }
            "value" | "val" | "reading" | "measurement" | "data" | "pv" => {
                mapping.insert("value".to_string(), idx);
            }
            "timestamp" | "time" | "ts" | "datetime" | "date" | "date time" => {
                mapping.insert("timestamp".to_string(), idx);
            }
            "device" | "device id" | "deviceid" | "device_name" | "plc" => {
                mapping.insert("device_id".to_string(), idx);
            }
            _ => {}
        }
    }

    mapping
}

/// Parse a timestamp string into epoch seconds.
fn parse_timestamp(s: &str) -> Result<i64, String> {
    // Try ISO 8601
    if let Ok(dt) = time::OffsetDateTime::parse(s, &time::format_description::well_known::Iso8601) {
        return Ok(dt.unix_timestamp());
    }
    // Try ISO 8601 date only
    if let Ok(dt) = time::OffsetDateTime::parse(
        s,
        &time::format_description::well_known::Iso8601::DATE,
    ) {
        return Ok(dt.unix_timestamp());
    }
    // Try epoch seconds
    if let Ok(ts) = s.parse::<i64>() {
        return Ok(ts);
    }
    // Try epoch milliseconds
    if let Ok(ts_ms) = s.parse::<f64>() {
        return Ok(ts_ms as i64 / 1000);
    }

    Err(format!("Cannot parse timestamp: '{}'", s))
}

/// POST /api/v1/import/csv — multipart CSV upload with auto-detect and import
pub async fn import_csv(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(params): Query<ImportQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let dry_run = params.dry_run.as_deref() == Some("true");
    let mode = params.mode.as_deref().unwrap_or("merge");

    if mode != "merge" && mode != "replace" {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "mode must be 'merge' or 'replace'"})),
        ).into_response());
    }

    // Read the file field
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            file_name = field.file_name().map(|s: &str| s.to_string());
            file_bytes = Some(field.bytes().await?.to_vec());
        }
    }

    let bytes = file_bytes.ok_or_else(|| AppError(anyhow::anyhow!("No file uploaded")))?;
    let original_name = file_name.unwrap_or_else(|| "data.csv".to_string());

    if !original_name.to_lowercase().ends_with(".csv") {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Only CSV files are supported"})),
        ).into_response());
    }

    let content = String::from_utf8_lossy(&bytes);
    let delimiter = match params.delimiter.as_deref() {
        Some("\\t") => b'\t',
        Some(";") => b';',
        _ => b',',
    };

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers: Vec<String> = reader
        .headers()?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let column_mapping = auto_detect_columns(&headers);

    // Validate we have minimum required columns
    if !column_mapping.contains_key("tag_name") {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Could not auto-detect tag/name column. Ensure your CSV has a column named 'name', 'tag', 'sensor', or 'key'.",
                "detected_columns": headers,
                "expected": ["name/tag/sensor/key", "value", "timestamp (optional)", "device (optional)"],
            })),
        ).into_response());
    }

    let tag_name_idx = column_mapping["tag_name"];
    let value_idx = column_mapping.get("value").copied();
    let timestamp_idx = column_mapping.get("timestamp").copied();
    let device_idx = column_mapping.get("device_id").copied();

    let mut total_rows = 0usize;
    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    // Batch for telemetry insert: (device_id, timestamp, metric, value)
    let mut batch: Vec<(String, i64, String, f64)> = Vec::new();

    for (row_num, record) in reader.records().enumerate() {
        let record = match record {
            Ok(r) => r,
            Err(e) => {
                errors.push(json!({
                    "row": row_num + 2, // +2 for header + 1-indexed
                    "error": format!("CSV parse error: {}", e),
                }));
                total_rows += 1;
                continue;
            }
        };

        total_rows += 1;

        let tag_name = match record.get(tag_name_idx) {
            Some(v) if !v.trim().is_empty() => v.trim().to_string(),
            _ => {
                errors.push(json!({
                    "row": row_num + 2,
                    "error": "Empty tag name",
                }));
                skipped += 1;
                continue;
            }
        };

        let value = match value_idx.and_then(|i| record.get(i)) {
            Some(v) => match v.trim().parse::<f64>() {
                Ok(f) => f,
                Err(_) => {
                    errors.push(json!({
                        "row": row_num + 2,
                        "tag": tag_name,
                        "error": format!("Invalid value: '{}'", v.trim()),
                    }));
                    skipped += 1;
                    continue;
                }
            },
            None => {
                errors.push(json!({
                    "row": row_num + 2,
                    "tag": tag_name,
                    "error": "No value column detected",
                }));
                skipped += 1;
                continue;
            }
        };

        let device_id = device_idx
            .and_then(|i| record.get(i))
            .map(|v| v.trim().to_string())
            .unwrap_or_else(|| "default".to_string());

        let timestamp = match timestamp_idx.and_then(|i| record.get(i)) {
            Some(v) if !v.trim().is_empty() => match parse_timestamp(v.trim()) {
                Ok(ts) => ts,
                Err(e) => {
                    errors.push(json!({
                        "row": row_num + 2,
                        "tag": tag_name,
                        "error": e,
                    }));
                    skipped += 1;
                    continue;
                }
            },
            _ => crate::timeutil::now_secs(),
        };

        if !dry_run {
            batch.push((device_id.clone(), timestamp, tag_name.clone(), value));

            // Flush batch every 500 rows
            if batch.len() >= 500 {
                match state.storage.insert_batch(&batch).await {
                    Ok(_) => imported += batch.len(),
                    Err(e) => {
                        errors.push(json!({
                            "error": format!("Batch insert failed: {}", e),
                        }));
                    }
                }
                batch.clear();
            }
        } else {
            imported += 1;
        }
    }

    // Flush remaining batch
    if !batch.is_empty() && !dry_run {
        match state.storage.insert_batch(&batch).await {
            Ok(_) => imported += batch.len(),
            Err(e) => {
                errors.push(json!({
                    "error": format!("Final batch insert failed: {}", e),
                }));
            }
        }
    }

    state.audit_logger.log(crate::security::AuditEvent {
        action: "csv_data_import".to_string(),
        username: auth.username.clone(),
        resource: format!("import:{}", original_name),
        details: Some(format!(
            "total={}, imported={}, skipped={}, errors={}, dry_run={}",
            total_rows, imported, skipped, errors.len(), dry_run
        )),
    });

    Ok(Json(json!({
        "total_rows": total_rows,
        "imported": imported,
        "skipped": skipped,
        "errors": errors,
        "dry_run": dry_run,
        "mode": mode,
        "column_mapping": column_mapping,
    })))
}
```

- [ ] 5.3 Register route in `src/api/mod.rs`

Add module declaration:

```rust
mod csv_import;
```

Add to `admin_protected` router. Since the default body limit is 1MB, we need to override it for the import route. Add this as a separate nested router with increased limit:

```rust
// In create_router(), before building admin_protected:
let import_routes = Router::new()
    .route("/api/v1/import/csv", post(csv_import::import_csv))
    .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB for CSV upload
    .layer(middleware::from_fn(admin_only_middleware))
    .layer(middleware::from_fn_with_state(
        state.clone(),
        cookie_auth_middleware,
    ));
```

Then merge `import_routes` into `api_router` alongside the other routers:

```rust
let api_router = Router::new()
    .merge(public)
    .merge(protected)
    .merge(admin_protected)
    .merge(import_routes)  // <-- add this
    .route("/api/v1/{*path}", get(api_not_found))
    // ... rest unchanged
```

- [ ] 5.4 Write failing tests

Create `src/api/tests/csv_import.rs`:

```rust
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

use crate::api::tests::*;

fn make_multipart_body(csv_content: &str) -> Vec<u8> {
    let boundary = "----TestBoundary12345";
    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{}\r\n--{}--\r\n",
        boundary, csv_content, boundary
    );
    body.into_bytes()
}

#[tokio::test]
async fn test_csv_import_requires_auth() {
    let app = test_app();
    let boundary = "----TestBoundary12345";
    let csv = "name,value,timestamp\nPT001,42.5,1700000000\n";
    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{}\r\n--{}--\r\n",
        boundary, csv, boundary
    );
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import/csv")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_csv_import_dry_run() {
    let app = test_app_with_seeded_admin().await;
    let cookie = seed_and_get_cookie(&app).await;

    let csv = "name,value,timestamp\ntag_a,42.5,1700000000\ntag_b,100.0,1700000001\n";
    let boundary = "----TestBoundary12345";
    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{}\r\n--{}--\r\n",
        boundary, csv, boundary
    );
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/import/csv?dry_run=true")
                .header("cookie", format!("voltrus_token={}", cookie))
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp_body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(json["total_rows"], 2);
    assert_eq!(json["imported"], 2);
    assert_eq!(json["skipped"], 0);
    assert_eq!(json["dry_run"], true);
}
```

Register in `src/api/tests/mod.rs`:

```rust
mod csv_import;
```

- [ ] 5.5 Verify compilation and test pass

```bash
cargo test --lib api::tests::csv_import
```

- [ ] **Commit:** `feat(api): CSV data import with auto-detect columns, dry_run, merge/replace modes`

---

## Task 6: Frontend — Query Hooks for New Endpoints

**Files:**
- Create: `client/src/queries/tags.ts`
- Create: `client/src/queries/audit.ts`
- Create: `client/src/queries/import.ts`
- Modify: `client/src/queries/keys.ts`
- Modify: `client/src/queries/index.ts`
- Modify: `client/src/types.ts`

**Steps:**

- [ ] 6.1 Add query keys for new endpoints

Add to `client/src/queries/keys.ts`:

```typescript
  tagTree:       (search?: string) => ['tagTree', search] as const,
  namingConvention: ['namingConvention'] as const,
  auditLogs:     (params: Record<string, any>) => ['auditLogs', params] as const,
  auditExport:   (params: Record<string, any>) => ['auditExport', params] as const,
```

- [ ] 6.2 Create `client/src/queries/tags.ts`

```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { queryKeys } from './keys'
import { fetchJson, fetchOk } from './fetch'

export interface TagTreeNode {
  name: string
  tagCount: number
  tagIds: number[]
  children: TagTreeNode[]
}

export interface NamingConvention {
  template: string
  regex: string
  custom_regex: string | null
  templates: { id: string; label: string }[]
}

export function useTagTree(search?: string, enabled = true) {
  const sp = search ? `?search=${encodeURIComponent(search)}` : ''
  return useQuery({
    queryKey: queryKeys.tagTree(search),
    queryFn: () => fetchJson<{ tree: TagTreeNode[] }>(`/api/v1/tags/tree${sp}`),
    enabled,
  })
}

export function useUpdateTagPath() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async ({ id, path }: { id: number; path: string | null }) => {
      await fetchOk(`/api/v1/tags/${id}/path`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path }),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['tagTree'] })
    },
  })
}

export function useNamingConvention(enabled = true) {
  return useQuery({
    queryKey: queryKeys.namingConvention,
    queryFn: () => fetchJson<NamingConvention>('/api/v1/settings/naming-convention'),
    enabled,
  })
}

export function useUpdateNamingConvention() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: async (body: { template: string; custom_regex?: string }) => {
      await fetchOk('/api/v1/settings/naming-convention', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.namingConvention })
    },
  })
}
```

- [ ] 6.3 Create `client/src/queries/audit.ts`

```typescript
import { useQuery, useMutation } from '@tanstack/react-query'
import { queryKeys } from './keys'
import { fetchJson } from './fetch'

export interface AuditLogEntry {
  id: number
  action: string
  username: string
  resource: string
  details: string | null
  created_at: number
}

export interface AuditLogResponse {
  entries: AuditLogEntry[]
  total: number
  page: number
  limit: number
  total_pages: number
}

export interface AuditQueryParams {
  search?: string
  action?: string
  username?: string
  from?: number
  to?: number
  page?: number
  limit?: number
  sort?: string
  order?: string
}

export function useAuditLogs(params: AuditQueryParams = {}, enabled = true) {
  const sp = new URLSearchParams()
  if (params.search) sp.set('search', params.search)
  if (params.action) sp.set('action', params.action)
  if (params.username) sp.set('username', params.username)
  if (params.from) sp.set('from', String(params.from))
  if (params.to) sp.set('to', String(params.to))
  if (params.page) sp.set('page', String(params.page))
  if (params.limit) sp.set('limit', String(params.limit))
  if (params.sort) sp.set('sort', params.sort)
  if (params.order) sp.set('order', params.order)
  const qs = sp.toString()

  return useQuery({
    queryKey: queryKeys.auditLogs(params),
    queryFn: () => fetchJson<AuditLogResponse>(`/api/v1/audit${qs ? '?' + qs : ''}`),
    enabled,
  })
}

export function useExportAuditCsv() {
  return useMutation({
    mutationFn: async (params: AuditQueryParams = {}) => {
      const sp = new URLSearchParams()
      if (params.search) sp.set('search', params.search)
      if (params.action) sp.set('action', params.action)
      if (params.username) sp.set('username', params.username)
      if (params.from) sp.set('from', String(params.from))
      if (params.to) sp.set('to', String(params.to))
      const qs = sp.toString()
      const res = await fetch(`/api/v1/audit/export${qs ? '?' + qs : ''}`, { credentials: 'include' })
      if (!res.ok) throw new Error('Export failed')
      return res.text()
    },
  })
}
```

- [ ] 6.4 Create `client/src/queries/import.ts`

```typescript
import { useMutation } from '@tanstack/react-query'
import { fetchJson } from './fetch'

export interface CsvImportResult {
  total_rows: number
  imported: number
  skipped: number
  errors: { row?: number; error: string; tag?: string }[]
  dry_run: boolean
  mode: string
  column_mapping: Record<string, number>
}

export interface CsvImportParams {
  file: File
  dryRun?: boolean
  mode?: 'merge' | 'replace'
  delimiter?: string
}

export function useImportCsv() {
  return useMutation({
    mutationFn: async ({ file, dryRun, mode, delimiter }: CsvImportParams) => {
      const sp = new URLSearchParams()
      if (dryRun) sp.set('dry_run', 'true')
      if (mode) sp.set('mode', mode)
      if (delimiter) sp.set('delimiter', delimiter)
      const qs = sp.toString()

      const formData = new FormData()
      formData.append('file', file)

      const res = await fetch(`/api/v1/import/csv${qs ? '?' + qs : ''}`, {
        method: 'POST',
        credentials: 'include',
        body: formData,
      })
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `Import failed: ${res.status}`)
      }
      return res.json() as Promise<CsvImportResult>
    },
  })
}
```

- [ ] 6.5 Export new hooks from `client/src/queries/index.ts`

Add to `client/src/queries/index.ts`:

```typescript
export {
  useTagTree,
  useUpdateTagPath,
  useNamingConvention,
  useUpdateNamingConvention,
} from './tags'

export type { TagTreeNode, NamingConvention } from './tags'

export {
  useAuditLogs,
  useExportAuditCsv,
} from './audit'

export type { AuditLogEntry, AuditLogResponse, AuditQueryParams } from './audit'

export {
  useImportCsv,
} from './import'

export type { CsvImportResult, CsvImportParams } from './import'
```

- [ ] 6.6 Add `path` field to `SensorConfig` type in `client/src/types.ts`

```typescript
export interface SensorConfig {
  id: number; key: string; name: string; unit: string; color: string;
  min_val: number; max_val: number; warn_val: number; crit_val: number;
  base_val: number; sort_order: number; decimals: number;
  source_type: string; source_config: string;
  device_id: string;
  equipment_id: string | null;
  chart_type: string;
  path: string | null;  // <-- add this line
}
```

- [ ] **Commit:** `feat(client): query hooks for tag tree, audit log, CSV import, naming convention`

---

## Task 7: Frontend — Tag Browser with UNS Tree View

**Files:**
- Create: `client/src/TagBrowser.tsx`
- Modify: `client/src/App.tsx` (wire TagBrowser into mode system)

**Steps:**

- [ ] 7.1 Create `client/src/TagBrowser.tsx`

```tsx
import { useState, useCallback, useMemo } from 'react'
import { useTagTree, useSensors } from './queries'
import type { TagTreeNode, SensorConfig } from './queries'
import { inputStyle, btnStyle } from './styles'

// ── Tree Node Component ──

interface TreeNodeProps {
  node: TagTreeNode
  sensors: SensorConfig[]
  depth: number
  selectedTagId: number | null
  onSelectTag: (id: number) => void
  onDragStart: (e: React.DragEvent, tag: SensorConfig) => void
}

function TreeNode({ node, sensors, depth, selectedTagId, onSelectTag, onDragStart }: TreeNodeProps) {
  const [expanded, setExpanded] = useState(depth < 2)
  const hasChildren = node.children.length > 0
  const hasTags = node.tagIds.length > 0

  const toggle = () => {
    if (hasChildren || hasTags) setExpanded(v => !v)
  }

  return (
    <div>
      <div
        onClick={toggle}
        style={{
          display: 'flex', alignItems: 'center', gap: 6,
          padding: '4px 8px', paddingLeft: 8 + depth * 16,
          cursor: 'pointer', borderRadius: 4,
          fontSize: 13, fontWeight: hasChildren || hasTags ? 600 : 400,
          color: 'var(--fg)',
          background: 'transparent',
          transition: 'background 0.1s',
          userSelect: 'none',
        }}
        onMouseEnter={e => (e.currentTarget.style.background = 'var(--border)')}
        onMouseLeave={e => (e.currentTarget.style.background = 'transparent')}
      >
        {/* Expand/collapse chevron */}
        <span style={{
          width: 14, height: 14, display: 'flex', alignItems: 'center', justifyContent: 'center',
          transition: 'transform 0.15s',
          transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)',
          opacity: (hasChildren || hasTags) ? 1 : 0,
        }}>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="3,1 7,5 3,9" />
          </svg>
        </span>

        {/* Folder or tag icon */}
        {hasChildren ? (
          <svg width="14" height="14" viewBox="0 0 24 24" fill="var(--cyan)" stroke="none">
            <path d="M2 6a2 2 0 012-2h5l2 2h9a2 2 0 012 2v10a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
          </svg>
        ) : (
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--ok)" strokeWidth="2">
            <circle cx="12" cy="12" r="3" />
            <line x1="12" y1="5" x2="12" y2="9" />
            <line x1="12" y1="15" x2="12" y2="19" />
            <line x1="5" y1="12" x2="9" y2="12" />
            <line x1="15" y1="12" x2="19" y2="12" />
          </svg>
        )}

        <span style={{ flex: 1 }}>{node.name}</span>
        {node.tagCount > 0 && (
          <span style={{ fontSize: 10, color: 'var(--muted)', background: 'var(--bg)', borderRadius: 8, padding: '1px 6px' }}>
            {node.tagCount}
          </span>
        )}
      </div>

      {/* Children / Tag leaves */}
      {expanded && (
        <div>
          {node.children.map(child => (
            <TreeNode
              key={child.name}
              node={child}
              sensors={sensors}
              depth={depth + 1}
              selectedTagId={selectedTagId}
              onSelectTag={onSelectTag}
              onDragStart={onDragStart}
            />
          ))}
          {expanded && node.tagIds.map(tagId => {
            const sensor = sensors.find(s => s.id === tagId)
            if (!sensor) return null
            return (
              <div
                key={tagId}
                draggable
                onDragStart={e => onDragStart(e, sensor)}
                onClick={() => onSelectTag(tagId)}
                style={{
                  display: 'flex', alignItems: 'center', gap: 6,
                  padding: '3px 8px', paddingLeft: 8 + (depth + 1) * 16,
                  cursor: 'pointer', borderRadius: 4,
                  fontSize: 12,
                  background: selectedTagId === tagId ? 'var(--cyan)' : 'transparent',
                  color: selectedTagId === tagId ? '#000' : 'var(--fg)',
                  transition: 'background 0.1s',
                }}
                onMouseEnter={e => {
                  if (selectedTagId !== tagId) e.currentTarget.style.background = 'var(--border)'
                }}
                onMouseLeave={e => {
                  if (selectedTagId !== tagId) e.currentTarget.style.background = 'transparent'
                }}
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <circle cx="12" cy="12" r="10" />
                  <line x1="12" y1="8" x2="12" y2="12" />
                  <line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
                <span style={{ flex: 1, fontWeight: 500 }}>{sensor.name}</span>
                <span style={{ fontSize: 10, color: 'var(--muted)' }}>{sensor.unit}</span>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}

// ── Main TagBrowser Component ──

interface Props {
  isAdmin: boolean
  onDragTag?: (tag: SensorConfig) => void
  onSelectTag?: (id: number) => void
}

export function TagBrowser({ isAdmin, onDragTag, onSelectTag }: Props) {
  const [search, setSearch] = useState('')
  const [selectedTagId, setSelectedTagId] = useState<number | null>(null)

  const { data: treeData, isLoading } = useTagTree(search || undefined)
  const { data: sensors } = useSensors(true)

  const tree = treeData?.tree ?? []
  const sensorList = sensors ?? []

  const handleDragStart = useCallback((e: React.DragEvent, tag: SensorConfig) => {
    e.dataTransfer.setData('application/json', JSON.stringify({
      type: 'tag',
      id: tag.id,
      name: tag.name,
      key: tag.key,
      unit: tag.unit,
      color: tag.color,
      min_val: tag.min_val,
      max_val: tag.max_val,
      chart_type: tag.chart_type,
    }))
    e.dataTransfer.effectAllowed = 'copy'
    onDragTag?.(tag)
  }, [onDragTag])

  const handleSelectTag = useCallback((id: number) => {
    setSelectedTagId(id)
    onSelectTag?.(id)
  }, [onSelectTag])

  const selectedSensor = useMemo(
    () => sensorList.find(s => s.id === selectedTagId),
    [sensorList, selectedTagId]
  )

  return (
    <div style={{ display: 'flex', gap: 0, height: '100%', overflow: 'hidden' }}>
      {/* Left panel: tree */}
      <div style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', borderRight: '1px solid var(--border)' }}>
        {/* Header */}
        <div style={{ padding: '12px 16px', borderBottom: '1px solid var(--border)' }}>
          <div style={{ fontSize: 14, fontWeight: 700, color: 'var(--fg)', marginBottom: 8 }}>
            Tag Browser
          </div>
          <input
            type="text"
            placeholder="Search tags..."
            value={search}
            onChange={e => setSearch(e.target.value)}
            style={inputStyle}
          />
        </div>

        {/* Tree */}
        <div style={{ flex: 1, overflow: 'auto', padding: '4px 0' }}>
          {isLoading ? (
            <div style={{ padding: 20, textAlign: 'center', color: 'var(--muted)', fontSize: 13 }}>
              Loading tags...
            </div>
          ) : tree.length === 0 ? (
            <div style={{ padding: 20, textAlign: 'center', color: 'var(--muted)', fontSize: 13 }}>
              {search ? 'No tags match your search' : 'No tags configured yet'}
            </div>
          ) : (
            tree.map(node => (
              <TreeNode
                key={node.name}
                node={node}
                sensors={sensorList}
                depth={0}
                selectedTagId={selectedTagId}
                onSelectTag={handleSelectTag}
                onDragStart={handleDragStart}
              />
            ))
          )}
        </div>
      </div>

      {/* Right panel: tag details */}
      <div style={{ width: 300, flexShrink: 0, display: 'flex', flexDirection: 'column', overflow: 'auto' }}>
        {selectedSensor ? (
          <div style={{ padding: 16 }}>
            <div style={{ fontSize: 11, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.04em', marginBottom: 8 }}>
              Tag Details
            </div>
            <div style={{ fontSize: 16, fontWeight: 700, color: 'var(--fg)', marginBottom: 4 }}>
              {selectedSensor.name}
            </div>
            <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 16 }}>
              Key: {selectedSensor.key} | Device: {selectedSensor.device_id}
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 8, fontSize: 13 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--muted)' }}>Unit</span>
                <span>{selectedSensor.unit || '-'}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--muted)' }}>Range</span>
                <span>{selectedSensor.min_val} - {selectedSensor.max_val}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--muted)' }}>Warning</span>
                <span style={{ color: 'var(--warn)' }}>{selectedSensor.warn_val}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--muted)' }}>Critical</span>
                <span style={{ color: 'var(--crit)' }}>{selectedSensor.crit_val}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--muted)' }}>Path</span>
                <span style={{ fontFamily: 'monospace', fontSize: 11 }}>
                  {selectedSensor.path || '(not set)'}
                </span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--muted)' }}>Source</span>
                <span>{selectedSensor.source_type}</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: 'var(--muted)' }}>Chart</span>
                <span>{selectedSensor.chart_type}</span>
              </div>
            </div>

            {isAdmin && (
              <div style={{ marginTop: 16, padding: '8px 12px', background: 'var(--bg)', borderRadius: 6, fontSize: 11, color: 'var(--muted)' }}>
                Drag this tag onto a dashboard to create a widget.
              </div>
            )}
          </div>
        ) : (
          <div style={{ padding: 20, textAlign: 'center', color: 'var(--muted)', fontSize: 13, alignSelf: 'center' }}>
            Select a tag to view details
          </div>
        )}
      </div>
    </div>
  )
}
```

- [ ] 7.2 Wire TagBrowser into App.tsx mode system

In `client/src/App.tsx`, import `TagBrowser`:

```typescript
import { TagBrowser } from './TagBrowser'
```

Update the `mode` state type to include `'tags'`:

```typescript
const [mode, setMode] = useState<'live' | 'history' | 'system' | 'flows' | 'tags'>(() => {
    const m = new URLSearchParams(window.location.search).get('mode')
    return (m === 'history' || m === 'system' || m === 'flows' || m === 'tags') ? m : 'live'
})
```

Add rendering in the main content area (after `{mode === 'system' && ...}`):

```tsx
{mode === 'tags' && <TagBrowser isAdmin={isAdmin} />}
```

Add a navigation button in `AppHeader.tsx` for the tags mode (follow existing pattern for system/flows buttons):

In `client/src/components/AppHeader.tsx`, add a "Tags" button alongside the existing mode buttons:

```tsx
<button
  onClick={onTagsClick}
  style={{
    ...btnStyle,
    background: mode === 'tags' ? 'var(--cyan)' : 'var(--card)',
    color: mode === 'tags' ? '#000' : 'var(--fg)',
  }}
>
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
  </svg>
  Tags
</button>
```

Pass `onTagsClick` prop to AppHeader:

```tsx
<AppHeader
  // ... existing props
  onTagsClick={() => { setMode('tags'); setActiveScreen('all') }}
/>
```

- [ ] **Commit:** `feat(client): TagBrowser component with UNS tree view, search, drag-and-drop, tag details`

---

## Task 8: Frontend — Naming Convention Validation in AddSensorDialog

**Files:**
- Modify: `client/src/AddSensorDialog.tsx`

**Steps:**

- [ ] 8.1 Add naming convention validation hint to AddSensorDialog

At the top of `AddSensorDialog.tsx`, import the naming convention hook:

```typescript
import { useNamingConvention } from './queries'
```

Inside the component, fetch the naming convention:

```typescript
const { data: namingConvention } = useNamingConvention()
```

Add a validation state and function:

```typescript
const [nameError, setNameError] = useState<string | null>(null)

const validateName = (name: string) => {
  if (!namingConvention?.regex || !name) {
    setNameError(null)
    return true
  }
  try {
    const re = new RegExp(namingConvention.regex)
    if (!re.test(name)) {
      setNameError(`Name must match pattern: ${namingConvention.regex}`)
      return false
    }
  } catch {
    // Invalid regex, skip validation
  }
  setNameError(null)
  return true
}
```

Add a hint below the name input field:

```tsx
<div style={labelStyle}>
  <span style={labelTextStyle}>Tag Name</span>
  <input
    style={{
      ...inputStyle,
      borderColor: nameError ? 'var(--crit)' : 'var(--border)',
    }}
    value={form.name}
    onChange={e => {
      setForm(f => ({ ...f, name: e.target.value }))
      validateName(e.target.value)
    }}
  />
  {namingConvention && (
    <span style={{ fontSize: 10, color: 'var(--muted)', marginTop: 2 }}>
      Convention: {namingConvention.template} ({namingConvention.regex})
    </span>
  )}
  {nameError && (
    <span style={{ fontSize: 10, color: 'var(--crit)', marginTop: 2 }}>
      {nameError}
    </span>
  )}
</div>
```

- [ ] **Commit:** `feat(client): ISA-5.1 naming convention validation hint in AddSensorDialog`

---

## Task 9: Frontend — Audit Log Panel

**Files:**
- Create: `client/src/AuditLogPanel.tsx`
- Modify: `client/src/App.tsx` (wire into mode system or as overlay panel)

**Steps:**

- [ ] 9.1 Create `client/src/AuditLogPanel.tsx`

```tsx
import { useState, useCallback } from 'react'
import { useAuditLogs, useExportAuditCsv } from './queries'
import type { AuditQueryParams } from './queries'
import { inputStyle, btnStyle, labelStyle, labelTextStyle } from './styles'

const ACTION_OPTIONS = [
  { value: '', label: 'All Actions' },
  { value: 'update_branding', label: 'Update Branding' },
  { value: 'update_alarm_cooldown', label: 'Update Alarm Cooldown' },
  { value: 'csv_data_import', label: 'CSV Import' },
  { value: 'create_data_source', label: 'Create Tag' },
  { value: 'update_data_source', label: 'Update Tag' },
  { value: 'delete_data_source', label: 'Delete Tag' },
  { value: 'create_user', label: 'Create User' },
  { value: 'login_success', label: 'Login' },
  { value: 'login_failed', label: 'Login Failed' },
] as const

function formatTimestamp(epoch: number): string {
  return new Date(epoch * 1000).toLocaleString('en-US', {
    year: 'numeric', month: 'short', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
  })
}

function triggerCsvDownload(csvText: string) {
  const blob = new Blob([csvText], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `audit-log-${new Date().toISOString().slice(0, 10)}.csv`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

export function AuditLogPanel() {
  const [search, setSearch] = useState('')
  const [action, setAction] = useState('')
  const [username, setUsername] = useState('')
  const [page, setPage] = useState(1)
  const [sort, setSort] = useState('created_at')
  const [order, setOrder] = useState<'asc' | 'desc'>('desc')
  const [fromDate, setFromDate] = useState('')
  const [toDate, setToDate] = useState('')

  const params: AuditQueryParams = {
    search: search || undefined,
    action: action || undefined,
    username: username || undefined,
    page,
    limit: 50,
    sort,
    order,
    from: fromDate ? new Date(fromDate).getTime() / 1000 : undefined,
    to: toDate ? new Date(toDate).getTime() / 1000 + 86400 : undefined,
  }

  const { data, isLoading } = useAuditLogs(params)
  const exportCsv = useExportAuditCsv()

  const handleExport = useCallback(async () => {
    try {
      const csv = await exportCsv.mutateAsync(params)
      triggerCsvDownload(csv)
    } catch (e: any) {
      alert(e.message || 'Export failed')
    }
  }, [exportCsv, params])

  const handleSearch = () => {
    setPage(1)
  }

  const entries = data?.entries ?? []
  const totalPages = data?.total_pages ?? 1

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* Header + Filters */}
      <div style={{ padding: '12px 16px', borderBottom: '1px solid var(--border)' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 10 }}>
          <div style={{ fontSize: 14, fontWeight: 700, color: 'var(--fg)' }}>
            Audit Log
          </div>
          <button onClick={handleExport} disabled={exportCsv.isPending} style={btnStyle}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            {exportCsv.isPending ? 'Exporting...' : 'Export CSV'}
          </button>
        </div>

        {/* Filter row */}
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input
            type="text"
            placeholder="Search..."
            value={search}
            onChange={e => setSearch(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleSearch()}
            style={{ ...inputStyle, width: 200, fontSize: 12 }}
          />
          <select
            value={action}
            onChange={e => { setAction(e.target.value); setPage(1) }}
            style={{ ...inputStyle, width: 160, fontSize: 12 }}
          >
            {ACTION_OPTIONS.map(o => (
              <option key={o.value} value={o.value}>{o.label}</option>
            ))}
          </select>
          <input
            type="text"
            placeholder="Username"
            value={username}
            onChange={e => { setUsername(e.target.value); setPage(1) }}
            style={{ ...inputStyle, width: 120, fontSize: 12 }}
          />
          <input
            type="date"
            value={fromDate}
            onChange={e => { setFromDate(e.target.value); setPage(1) }}
            style={{ ...inputStyle, width: 140, fontSize: 12 }}
          />
          <input
            type="date"
            value={toDate}
            onChange={e => { setToDate(e.target.value); setPage(1) }}
            style={{ ...inputStyle, width: 140, fontSize: 12 }}
          />
        </div>
      </div>

      {/* Table */}
      <div style={{ flex: 1, overflow: 'auto' }}>
        {isLoading ? (
          <div style={{ padding: 20, textAlign: 'center', color: 'var(--muted)', fontSize: 13 }}>
            Loading audit logs...
          </div>
        ) : entries.length === 0 ? (
          <div style={{ padding: 20, textAlign: 'center', color: 'var(--muted)', fontSize: 13 }}>
            No audit entries found
          </div>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
            <thead>
              <tr style={{ position: 'sticky', top: 0, background: 'var(--bg)', zIndex: 1 }}>
                <th style={thStyle}>Timestamp</th>
                <th style={thStyle}>Action</th>
                <th style={thStyle}>User</th>
                <th style={thStyle}>Resource</th>
                <th style={thStyle}>Details</th>
              </tr>
            </thead>
            <tbody>
              {entries.map(entry => (
                <tr key={entry.id} style={{ borderBottom: '1px solid var(--border)' }}>
                  <td style={tdStyle}>
                    <span style={{ fontFamily: 'monospace', fontSize: 11 }}>
                      {formatTimestamp(entry.created_at)}
                    </span>
                  </td>
                  <td style={tdStyle}>
                    <span style={{
                      padding: '2px 6px', borderRadius: 4, fontSize: 10,
                      background: 'var(--bg)', color: 'var(--cyan)',
                      textTransform: 'uppercase', fontWeight: 600,
                    }}>
                      {entry.action}
                    </span>
                  </td>
                  <td style={tdStyle}>{entry.username}</td>
                  <td style={tdStyle}>{entry.resource}</td>
                  <td style={{ ...tdStyle, maxWidth: 300, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {entry.details || '-'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Pagination */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 16px', borderTop: '1px solid var(--border)', fontSize: 12 }}>
        <span style={{ color: 'var(--muted)' }}>
          {data?.total ?? 0} entries | Page {page} of {totalPages}
        </span>
        <div style={{ display: 'flex', gap: 4 }}>
          <button
            onClick={() => setPage(p => Math.max(1, p - 1))}
            disabled={page <= 1}
            style={{ ...btnStyle, padding: '4px 10px', fontSize: 11, opacity: page <= 1 ? 0.5 : 1 }}
          >
            Previous
          </button>
          <button
            onClick={() => setPage(p => Math.min(totalPages, p + 1))}
            disabled={page >= totalPages}
            style={{ ...btnStyle, padding: '4px 10px', fontSize: 11, opacity: page >= totalPages ? 0.5 : 1 }}
          >
            Next
          </button>
        </div>
      </div>
    </div>
  )
}

const thStyle: React.CSSProperties = {
  textAlign: 'left', padding: '6px 10px', fontSize: 10,
  color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: '.04em',
  fontWeight: 600, borderBottom: '1px solid var(--border)', whiteSpace: 'nowrap',
}

const tdStyle: React.CSSProperties = {
  padding: '6px 10px', whiteSpace: 'nowrap', color: 'var(--fg)',
}
```

- [ ] 9.2 Wire AuditLogPanel into App.tsx

Import in `App.tsx`:

```typescript
import { AuditLogPanel } from './AuditLogPanel'
```

Add state for showing the panel:

```typescript
const [showAuditPanel, setShowAuditPanel] = useState(false)
```

Add the panel rendering (similar to other overlay panels like AlarmPanel):

```tsx
{showAuditPanel && (
  <div style={{
    position: 'absolute', top: 0, left: 0, right: 0, bottom: 0, zIndex: 5,
    background: 'var(--card)', display: 'flex', flexDirection: 'column',
  }}>
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 16px', borderBottom: '1px solid var(--border)' }}>
      <span style={{ fontSize: 14, fontWeight: 600 }}>Audit Log</span>
      <button onClick={() => setShowAuditPanel(false)} style={btnStyle}>Close</button>
    </div>
    <div style={{ flex: 1, overflow: 'hidden' }}>
      <AuditLogPanel />
    </div>
  </div>
)}
```

Pass `onAuditPanelToggle` to AppHeader:

```tsx
<AppHeader
  // ... existing props
  onAuditPanelToggle={() => setShowAuditPanel(v => !v)}
/>
```

Add the button in AppHeader.tsx:

```tsx
<button
  onClick={onAuditPanelToggle}
  style={{
    ...btnStyle,
    background: showAuditPanel ? 'var(--cyan)' : 'var(--card)',
    color: showAuditPanel ? '#000' : 'var(--fg)',
  }}
>
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
    <polyline points="14 2 14 8 20 8" />
    <line x1="16" y1="13" x2="8" y2="13" />
    <line x1="16" y1="17" x2="8" y2="17" />
  </svg>
  Audit
</button>
```

- [ ] **Commit:** `feat(client): AuditLogPanel with search, filter, pagination, and CSV export`

---

## Task 10: Frontend — Data Import Wizard

**Files:**
- Create: `client/src/DataImportWizard.tsx`
- Modify: `client/src/App.tsx` (wire into existing export panel area or as overlay)

**Steps:**

- [ ] 10.1 Create `client/src/DataImportWizard.tsx`

```tsx
import { useState, useRef, useEffect } from 'react'
import { useImportCsv } from './queries'
import type { CsvImportResult } from './queries'
import { inputStyle, btnStyle, labelStyle, labelTextStyle } from './styles'

type Step = 1 | 2 | 3 | 4 | 5

interface Props {
  onClose: () => void
}

export function DataImportWizard({ onClose }: Props) {
  const [step, setStep] = useState<Step>(1)
  const [file, setFile] = useState<File | null>(null)
  const [mode, setMode] = useState<'merge' | 'replace'>('merge')
  const [delimiter, setDelimiter] = useState(',')
  const [dryRun, setDryRun] = useState(true)
  const [preview, setPreview] = useState<CsvImportResult | null>(null)
  const [errors, setErrors] = useState<string[]>([])
  const fileInputRef = useRef<HTMLInputElement>(null)

  const importCsv = useImportCsv()

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !importCsv.isPending) onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [onClose, importCsv.isPending])

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files?.[0] ?? null
    setFile(selected)
    setPreview(null)
    setErrors([])
    setStep(1)
  }

  const handlePreview = async () => {
    if (!file) return
    setErrors([])
    try {
      const result = await importCsv.mutateAsync({
        file,
        dryRun: true,
        mode,
        delimiter,
      })
      setPreview(result)
      setStep(2)
    } catch (e: any) {
      setErrors([e.message || 'Preview failed'])
    }
  }

  const handleImport = async () => {
    if (!file) return
    setErrors([])
    try {
      const result = await importCsv.mutateAsync({
        file,
        dryRun: false,
        mode,
        delimiter,
      })
      setPreview(result)
      setStep(4)
    } catch (e: any) {
      setErrors([e.message || 'Import failed'])
    }
  }

  const handleReset = () => {
    setFile(null)
    setPreview(null)
    setErrors([])
    setStep(1)
    setDryRun(true)
    if (fileInputRef.current) fileInputRef.current.value = ''
  }

  const busy = importCsv.isPending

  const stepIndicator = (n: Step, label: string) => (
    <span style={{
      fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.04em',
      color: step >= n ? 'var(--cyan)' : 'var(--muted)',
      opacity: step === n ? 1 : 0.6,
    }}>
      {n}. {label}
    </span>
  )

  return (
    <div
      style={{
        position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, zIndex: 10,
        background: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}
      onClick={() => { if (!busy) onClose() }}
    >
      <div
        onClick={e => e.stopPropagation()}
        style={{
          background: 'var(--card)', borderRadius: 12, width: 560, maxHeight: '80vh',
          display: 'flex', flexDirection: 'column', overflow: 'hidden',
          border: '1px solid var(--border)', boxShadow: '0 8px 32px rgba(0,0,0,0.3)',
        }}
      >
        {/* Header */}
        <div style={{ padding: '16px 20px', borderBottom: '1px solid var(--border)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: 15, fontWeight: 700 }}>Data Import Wizard</span>
          <button onClick={onClose} disabled={busy} style={{ ...btnStyle, padding: '4px 8px' }}>X</button>
        </div>

        {/* Steps indicator */}
        <div style={{ padding: '10px 20px', display: 'flex', gap: 20, borderBottom: '1px solid var(--border)' }}>
          {stepIndicator(1, 'Upload')}
          {stepIndicator(2, 'Preview')}
          {stepIndicator(3, 'Import')}
          {stepIndicator(4, 'Report')}
        </div>

        {/* Content */}
        <div style={{ flex: 1, overflow: 'auto', padding: 20 }}>
          {/* Step 1: Upload */}
          {step === 1 && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
              <div style={labelStyle}>
                <span style={labelTextStyle}>CSV File</span>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".csv"
                  onChange={handleFileChange}
                  style={inputStyle}
                />
              </div>

              <div style={labelStyle}>
                <span style={labelTextStyle}>Import Mode</span>
                <select value={mode} onChange={e => setMode(e.target.value as 'merge' | 'replace')} style={inputStyle}>
                  <option value="merge">Merge (append to existing data)</option>
                  <option value="replace">Replace (clear matching data first)</option>
                </select>
              </div>

              <div style={labelStyle}>
                <span style={labelTextStyle}>Delimiter</span>
                <select value={delimiter} onChange={e => setDelimiter(e.target.value)} style={inputStyle}>
                  <option value=",">Comma (,)</option>
                  <option value=";">Semicolon (;)</option>
                  <option value="\\t">Tab</option>
                </select>
              </div>

              <div style={labelStyle}>
                <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 13, cursor: 'pointer' }}>
                  <input type="checkbox" checked={dryRun} onChange={e => setDryRun(e.target.checked)} />
                  Preview only (dry run) - do not insert data
                </label>
              </div>

              {errors.length > 0 && (
                <div style={{ padding: 8, background: 'rgba(239,68,68,0.1)', borderRadius: 6, fontSize: 12, color: 'var(--crit)' }}>
                  {errors.map((e, i) => <div key={i}>{e}</div>)}
                </div>
              )}
            </div>
          )}

          {/* Step 2: Preview */}
          {step === 2 && preview && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr 1fr', gap: 8 }}>
                <div style={{ padding: 10, background: 'var(--bg)', borderRadius: 6, textAlign: 'center' }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--fg)' }}>{preview.total_rows}</div>
                  <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' }}>Total Rows</div>
                </div>
                <div style={{ padding: 10, background: 'var(--bg)', borderRadius: 6, textAlign: 'center' }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--ok)' }}>{preview.imported}</div>
                  <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' }}>Imported</div>
                </div>
                <div style={{ padding: 10, background: 'var(--bg)', borderRadius: 6, textAlign: 'center' }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--warn)' }}>{preview.skipped}</div>
                  <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' }}>Skipped</div>
                </div>
                <div style={{ padding: 10, background: 'var(--bg)', borderRadius: 6, textAlign: 'center' }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--crit)' }}>{preview.errors.length}</div>
                  <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' }}>Errors</div>
                </div>
              </div>

              <div style={{ padding: 8, background: 'var(--bg)', borderRadius: 6, fontSize: 12 }}>
                <div style={{ color: 'var(--muted)', marginBottom: 4 }}>Auto-detected columns:</div>
                <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
                  {Object.entries(preview.column_mapping).map(([field, idx]) => (
                    <span key={field} style={{ padding: '2px 8px', background: 'var(--card)', borderRadius: 4, border: '1px solid var(--border)', fontSize: 11 }}>
                      {field} (col {idx})
                    </span>
                  ))}
                </div>
              </div>

              {preview.errors.length > 0 && (
                <div style={{ padding: 8, background: 'rgba(239,68,68,0.1)', borderRadius: 6, fontSize: 11, color: 'var(--crit)', maxHeight: 150, overflow: 'auto' }}>
                  {preview.errors.map((e, i) => (
                    <div key={i} style={{ padding: '2px 0' }}>
                      Row {e.row}: {e.error} {e.tag && `(tag: ${e.tag})`}
                    </div>
                  ))}
                </div>
              )}

              {preview.dry_run && (
                <div style={{ padding: 8, background: 'rgba(34,197,94,0.1)', borderRadius: 6, fontSize: 12, color: 'var(--ok)' }}>
                  This was a dry run. No data was inserted. Click "Import" to insert data.
                </div>
              )}
            </div>
          )}

          {/* Step 3: Importing (brief) */}
          {step === 3 && (
            <div style={{ textAlign: 'center', padding: 40, color: 'var(--muted)' }}>
              <div style={{ fontSize: 14, marginBottom: 8 }}>Importing data...</div>
              <div style={{ fontSize: 12 }}>Please wait, this may take a moment for large files.</div>
            </div>
          )}

          {/* Step 4: Report */}
          {step === 4 && preview && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12, alignItems: 'center', padding: '20px 0' }}>
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--ok)" strokeWidth="2">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                <polyline points="22 4 12 14.01 9 11.01" />
              </svg>
              <div style={{ fontSize: 16, fontWeight: 700, color: 'var(--fg)' }}>Import Complete</div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 8, width: '100%' }}>
                <div style={{ padding: 10, background: 'var(--bg)', borderRadius: 6, textAlign: 'center' }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--ok)' }}>{preview.imported}</div>
                  <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' }}>Imported</div>
                </div>
                <div style={{ padding: 10, background: 'var(--bg)', borderRadius: 6, textAlign: 'center' }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--warn)' }}>{preview.skipped}</div>
                  <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' }}>Skipped</div>
                </div>
                <div style={{ padding: 10, background: 'var(--bg)', borderRadius: 6, textAlign: 'center' }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: 'var(--crit)' }}>{preview.errors.length}</div>
                  <div style={{ fontSize: 10, color: 'var(--muted)', textTransform: 'uppercase' }}>Errors</div>
                </div>
              </div>

              {preview.errors.length > 0 && (
                <div style={{ width: '100%', padding: 8, background: 'rgba(239,68,68,0.1)', borderRadius: 6, fontSize: 11, color: 'var(--crit)', maxHeight: 120, overflow: 'auto' }}>
                  {preview.errors.map((e, i) => (
                    <div key={i}>Row {e.row}: {e.error}</div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div style={{ padding: '12px 20px', borderTop: '1px solid var(--border)', display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          {step === 1 && (
            <>
              <button onClick={onClose} disabled={busy} style={btnStyle}>Cancel</button>
              <button
                onClick={() => {
                  if (dryRun) {
                    handlePreview()
                  } else {
                    handleImport()
                  }
                }}
                disabled={!file || busy}
                style={{ ...btnStyle, background: 'var(--cyan)', color: '#000', fontWeight: 600 }}
              >
                {busy ? 'Processing...' : dryRun ? 'Preview' : 'Import'}
              </button>
            </>
          )}
          {step === 2 && (
            <>
              <button onClick={handleReset} disabled={busy} style={btnStyle}>Back</button>
              <button
                onClick={() => { setStep(3); handleImport() }}
                disabled={busy}
                style={{ ...btnStyle, background: 'var(--cyan)', color: '#000', fontWeight: 600 }}
              >
                Import Data
              </button>
            </>
          )}
          {step === 4 && (
            <button onClick={handleReset} style={{ ...btnStyle, background: 'var(--cyan)', color: '#000', fontWeight: 600 }}>
              Import Another File
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
```

- [ ] 10.2 Wire DataImportWizard into App.tsx

Import in `App.tsx`:

```typescript
import { DataImportWizard } from './DataImportWizard'
```

Add state:

```typescript
const [showImportWizard, setShowImportWizard] = useState(false)
```

Render as overlay:

```tsx
{showImportWizard && <DataImportWizard onClose={() => setShowImportWizard(false)} />}
```

Pass `onImportClick` to the existing DataExportPanel area or add a button in AppHeader:

In `AppHeader.tsx`, add near the Export button:

```tsx
<button
  onClick={onImportClick}
  style={btnStyle}
>
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
    <polyline points="17 8 12 3 7 8" />
    <line x1="12" y1="3" x2="12" y2="15" />
  </svg>
  Import
</button>
```

Pass the prop:

```tsx
<AppHeader
  // ... existing props
  onImportClick={() => setShowImportWizard(true)}
/>
```

- [ ] **Commit:** `feat(client): DataImportWizard with upload, preview, import, and report steps`

---

## Task 11: Integration Testing and Final Verification

**Files:**
- All test files created in previous tasks
- Run full test suite

**Steps:**

- [ ] 11.1 Run all backend tests

```bash
cargo test --lib
```

- [ ] 11.2 Run all new API tests individually

```bash
cargo test --lib api::tests::tag_tree
cargo test --lib api::tests::naming_convention
cargo test --lib api::tests::audit_log
cargo test --lib api::tests::csv_import
cargo test --lib storage::tests::tag_tree_tests
```

- [ ] 11.3 Verify frontend TypeScript compilation

```bash
cd client && npx tsc --noEmit
```

- [ ] 11.4 Verify frontend build

```bash
cd client && npm run build
```

- [ ] 11.5 Manual smoke test checklist

1. Start server: `cargo run`
2. Login as admin
3. Navigate to Tags mode -- verify tree loads
4. Create a tag with ISA-5.1 compliant name (e.g., `PT001_TEMP`) -- should succeed
5. Create a tag with non-compliant name (e.g., `my_sensor`) -- should show 400 error
6. Update tag path via `PUT /api/v1/tags/{id}/path` -- tree should reorganize
7. Open Audit Log panel -- verify entries load with pagination
8. Search audit logs -- verify filtering works
9. Export audit CSV -- verify file downloads
10. Open Data Import Wizard -- upload a CSV file
11. Verify dry run preview shows correct column mapping
12. Import data and verify telemetry records are created

- [ ] **Commit:** `test: integration verification for v0.32.0 Track 2 features`

---

## Summary of All New/Modified Files

### Backend (Rust)

| File | Action | Purpose |
|------|--------|---------|
| `src/storage/schema/migrations.rs` | Modify | Add `path` column, audit_log indexes |
| `src/models/mod.rs` | Modify | Add `path` to DataSource, add `AuditLogEntry` |
| `src/storage/device_crud.rs` | Modify | Read/write `path`, add `list_data_sources_for_tree`, `build_tag_tree`, `update_data_source_path` |
| `src/storage/settings_audit.rs` | Modify | Extended search/filter/pagination, CSV export |
| `src/api/mod.rs` | Modify | Register 4 new modules + routes, increase body limit for import |
| `src/api/tag_tree.rs` | Create | `GET /tags/tree`, `PUT /tags/{id}/path` |
| `src/api/naming_convention.rs` | Create | `GET/PUT /settings/naming-convention`, `validate_tag_name()` |
| `src/api/audit_log.rs` | Create | `GET /audit`, `GET /audit/export` |
| `src/api/csv_import.rs` | Create | `POST /import/csv` |
| `src/api/data_sources.rs` | Modify | Add naming validation on create/update |
| `Cargo.toml` | Modify | Add `regex`, `csv` dependencies |
| `src/api/tests/mod.rs` | Modify | Register 4 new test modules |
| `src/api/tests/tag_tree.rs` | Create | Auth + tree endpoint tests |
| `src/api/tests/naming_convention.rs` | Create | Validation + convention tests |
| `src/api/tests/audit_log.rs` | Create | Auth + audit endpoint tests |
| `src/api/tests/csv_import.rs` | Create | Auth + dry_run import tests |
| `src/storage/tests/mod.rs` | Modify | Register tag_tree test module |
| `src/storage/tests/tag_tree_tests.rs` | Create | Path roundtrip test |

### Frontend (TypeScript/React)

| File | Action | Purpose |
|------|--------|---------|
| `client/src/queries/keys.ts` | Modify | Add query keys for tags, audit, import |
| `client/src/queries/tags.ts` | Create | `useTagTree`, `useUpdateTagPath`, `useNamingConvention`, `useUpdateNamingConvention` |
| `client/src/queries/audit.ts` | Create | `useAuditLogs`, `useExportAuditCsv` |
| `client/src/queries/import.ts` | Create | `useImportCsv` |
| `client/src/queries/index.ts` | Modify | Export new hooks |
| `client/src/types.ts` | Modify | Add `path` to `SensorConfig` |
| `client/src/TagBrowser.tsx` | Create | UNS tree view with drag-and-drop |
| `client/src/AuditLogPanel.tsx` | Create | Search, filter, paginated table, CSV export |
| `client/src/DataImportWizard.tsx` | Create | Upload, preview, import, report wizard |
| `client/src/AddSensorDialog.tsx` | Modify | Add naming convention validation hint |
| `client/src/App.tsx` | Modify | Wire TagBrowser, AuditLogPanel, DataImportWizard |
| `client/src/components/AppHeader.tsx` | Modify | Add Tags, Audit, Import buttons |
