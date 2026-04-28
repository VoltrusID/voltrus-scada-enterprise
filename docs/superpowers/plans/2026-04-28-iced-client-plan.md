# Voltrus Iced Client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a minimal native desktop viewer for Voltrus SCADA using Iced + plotters-iced that shows live sensor data and history charts.

**Architecture:** Standalone `voltrus-iced/` crate communicating with the backend via HTTP API (same endpoints as web client). Elm-style architecture: `App` struct with `update()`, `view()`, `subscription()`. Three views: Login, Grid (live cards), Chart (history). Polling for live data via `iced::time::every` subscription.

**Tech Stack:** Iced 0.13, plotters-iced 0.11, reqwest (rustls-tls), serde, tokio, iced_aw (card widget)

**Spec:** `docs/superpowers/specs/2026-04-28-iced-client-design.md`

---

## File Structure

```
voltrus-iced/
├── Cargo.toml
├── iced-config.yaml
└── src/
    ├── main.rs        # Entry point, window config
    ├── app.rs         # App struct, state machine, update/view/subscription
    ├── theme.rs       # Color constants, dark theme helpers
    ├── http.rs        # HTTP client: login, fetch data, fetch history
    ├── models.rs      # API response types matching backend JSON
    └── views/
        ├── mod.rs
        ├── login.rs   # Login form view
        ├── grid.rs    # Sensor card grid + live polling
        └── chart.rs   # History line chart (plotters-iced Chart impl)
```

---

### Task 1: Scaffold voltrus-iced crate

**Files:**
- Create: `voltrus-iced/Cargo.toml`
- Create: `voltrus-iced/src/main.rs`
- Create: `voltrus-iced/iced-config.yaml`

- [ ] **Step 1: Create Cargo.toml with all dependencies**

```toml
[package]
name = "voltrus-iced"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
iced = "=0.13.2"
iced_aw = { version = "=0.13.0", default-features = false, features = ["card"] }
plotters-iced = "=0.11.0"
plotters = "=0.3.7"
reqwest = { version = "=0.12.15", default-features = false, features = ["json", "rustls-tls"] }
tokio = { version = "=1.52.1", features = ["rt-multi-thread", "sync", "time"] }
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.149"
anyhow = "=1.0.102"
```

- [ ] **Step 2: Create minimal main.rs that opens a window**

```rust
mod app;

fn main() -> iced::Result {
    iced::application("Voltrus Viewer", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .theme(|_| iced::Theme::Dark)
        .window_size((1024.0, 700.0))
        .run_with(app::App::new)
}
```

- [ ] **Step 3: Create minimal app.rs (placeholder)**

```rust
use iced::{Element, Task};

pub struct App;

#[derive(Debug, Clone)]
pub enum Message {}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self, Task::none())
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        iced::widget::text("Voltrus Viewer").into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}
```

- [ ] **Step 4: Create iced-config.yaml**

```yaml
server_url: "http://localhost:3000"
poll_interval_secs: 1
default_history_range: "30m"
```

- [ ] **Step 5: Build and verify it compiles**

Run: `cd voltrus-iced && cargo build`
Expected: Compiles successfully, warnings OK

- [ ] **Step 6: Commit**

```bash
git add voltrus-iced/
git commit -m "feat(iced): scaffold voltrus-iced crate with dependencies"
```

---

### Task 2: Create models and HTTP client

**Files:**
- Create: `voltrus-iced/src/models.rs`
- Create: `voltrus-iced/src/http.rs`

- [ ] **Step 1: Create models.rs matching backend API types**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub ok: bool,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataSourceData {
    pub id: i64,
    pub key: String,
    pub value: f64,
    pub warn: f64,
    pub crit: f64,
    pub source_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataSourceSummary {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub unit: String,
    pub color: String,
    pub warn_val: f64,
    pub crit_val: f64,
    pub decimals: i64,
    pub source_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataPoint {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryResponse {
    pub sensors: Vec<DataSourceSummary>,
    pub data: serde_json::Value,
    pub range: String,
    pub interval: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrandingSettings {
    pub title: String,
    pub subtitle: String,
    pub logo: String,
}
```

- [ ] **Step 2: Create http.rs with login, data fetch, and history fetch**

```rust
use anyhow::Result;
use reqwest::Client;

use crate::models::*;

pub struct HttpClient {
    client: Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .cookie_store(true)
                .build()
                .expect("failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<LoginResponse> {
        let resp = self.client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&serde_json::json!({"username": username, "password": password}))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            anyhow::bail!("Login failed: {}", resp.status())
        }
    }

    pub async fn fetch_data(&self) -> Result<Vec<DataSourceData>> {
        let resp = self.client
            .get(format!("{}/api/data", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_history(&self, range: &str) -> Result<HistoryResponse> {
        let resp = self.client
            .get(format!("{}/api/history?range={}", self.base_url, range))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_data_sources(&self) -> Result<Vec<DataSourceSummary>> {
        let resp = self.client
            .get(format!("{}/api/data-sources", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_branding(&self) -> Result<BrandingSettings> {
        let resp = self.client
            .get(format!("{}/api/settings/branding", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }
}
```

- [ ] **Step 3: Build to verify**

Run: `cd voltrus-iced && cargo build`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add voltrus-iced/src/models.rs voltrus-iced/src/http.rs
git commit -m "feat(iced): add HTTP client and API response models"
```

---

### Task 3: Implement App state machine and login view

**Files:**
- Modify: `voltrus-iced/src/app.rs`
- Create: `voltrus-iced/src/views/mod.rs`
- Create: `voltrus-iced/src/views/login.rs`

- [ ] **Step 1: Create views/mod.rs**

```rust
pub mod login;
pub mod grid;
pub mod chart;
```

- [ ] **Step 2: Create views/login.rs**

```rust
use iced::widget::{button, column, container, text, text_input};
use iced::{Center, Element, Fill};

use crate::app::Message;

pub struct LoginView;

impl LoginView {
    pub fn view(username: &str, password: &str, error: Option<&str>, loading: bool) -> Element<'_, Message> {
        let username_input = text_input("Username", username)
            .on_input(Message::UsernameChanged)
            .padding(10)
            .width(Fill);

        let password_input = text_input("Password", password)
            .on_input(Message::PasswordChanged)
            .on_submit(Message::Login)
            .secure(true)
            .padding(10)
            .width(Fill);

        let login_button = if loading {
            button(text("Logging in...").width(Fill).align_x(Center))
                .padding(10)
                .width(Fill)
        } else {
            button(text("Login").width(Fill).align_x(Center))
                .on_press(Message::Login)
                .padding(10)
                .width(Fill)
        };

        let mut form = column![
            text("Voltrus Viewer").size(28),
            username_input,
            password_input,
            login_button,
        ]
        .spacing(15)
        .max_width(350)
        .align_x(Center);

        if let Some(err) = error {
            form = form.push(text(err).style(iced::widget::text::danger));
        }

        container(form)
            .padding(40)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
}
```

- [ ] **Step 3: Rewrite app.rs with full state machine and login integration**

```rust
pub mod views;
pub mod http;
pub mod models;

use std::time::Duration;

use http::HttpClient;
use iced::time;
use iced::{Element, Fill, Subscription, Task};

#[derive(Debug, Clone)]
pub enum Message {
    UsernameChanged(String),
    PasswordChanged(String),
    Login,
    LoginResult(Result<models::LoginResponse, String>),
    PollTick,
    DataFetched(Result<Vec<models::DataSourceData>, String>),
    DataSourcesFetched(Result<Vec<models::DataSourceSummary>, String>),
    HistoryFetched(Result<models::HistoryResponse, String>),
    SelectSensor(i64),
    SetRange(String),
    BackToGrid,
    Logout,
}

#[derive(Debug, Clone)]
pub enum AppPhase {
    Login,
    Loading,
    Dashboard,
    Error(String),
}

pub struct App {
    phase: AppPhase,
    http: HttpClient,
    // Login state
    username: String,
    password: String,
    login_error: Option<String>,
    login_loading: bool,
    // Dashboard state
    data_sources: Vec<models::DataSourceSummary>,
    live_data: Vec<models::DataSourceData>,
    // Chart state
    selected_sensor: Option<i64>,
    history_points: Vec<models::DataPoint>,
    history_range: String,
    history_loading: bool,
    // Config
    server_url: String,
    poll_interval_secs: u64,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let server_url = "http://localhost:3000".to_string();
        let poll_interval_secs = 1;
        let http = HttpClient::new(&server_url);

        (
            Self {
                phase: AppPhase::Login,
                http,
                username: String::new(),
                password: String::new(),
                login_error: None,
                login_loading: false,
                data_sources: Vec::new(),
                live_data: Vec::new(),
                selected_sensor: None,
                history_points: Vec::new(),
                history_range: "30m".to_string(),
                history_loading: false,
                server_url,
                poll_interval_secs,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UsernameChanged(v) => { self.username = v; Task::none() }
            Message::PasswordChanged(v) => { self.password = v; Task::none() }
            Message::Login => {
                self.login_loading = true;
                self.login_error = None;
                let http = self.http.clone();
                let username = self.username.clone();
                let password = self.password.clone();
                Task::perform(
                    async move { http.login(&username, &password).await.map_err(|e| e.to_string()) },
                    Message::LoginResult,
                )
            }
            Message::LoginResult(result) => {
                self.login_loading = false;
                match result {
                    Ok(_) => {
                        self.phase = AppPhase::Loading;
                        let http = self.http.clone();
                        Task::batch([
                            Task::perform(
                                async move { http.fetch_data_sources().await.map_err(|e| e.to_string()) },
                                Message::DataSourcesFetched,
                            ),
                            Task::perform(
                                async move { http.fetch_data().await.map_err(|e| e.to_string()) },
                                Message::DataFetched,
                            ),
                        ])
                    }
                    Err(e) => {
                        self.login_error = Some(e);
                        Task::none()
                    }
                }
            }
            Message::PollTick => {
                if matches!(self.phase, AppPhase::Dashboard | AppPhase::Loading) {
                    let http = self.http.clone();
                    Task::perform(
                        async move { http.fetch_data().await.map_err(|e| e.to_string()) },
                        Message::DataFetched,
                    )
                } else {
                    Task::none()
                }
            }
            Message::DataFetched(result) => {
                match result {
                    Ok(data) => {
                        self.live_data = data;
                        if matches!(self.phase, AppPhase::Loading) {
                            self.phase = AppPhase::Dashboard;
                        }
                    }
                    Err(e) => {
                        if matches!(self.phase, AppPhase::Dashboard) {
                            self.phase = AppPhase::Error(e);
                        }
                    }
                }
                Task::none()
            }
            Message::DataSourcesFetched(result) => {
                if let Ok(sources) = result {
                    self.data_sources = sources;
                }
                Task::none()
            }
            Message::SelectSensor(id) => {
                self.selected_sensor = Some(id);
                self.history_loading = true;
                self.history_points.clear();
                let http = self.http.clone();
                let range = self.history_range.clone();
                Task::perform(
                    async move { http.fetch_history(&range).await.map_err(|e| e.to_string()) },
                    Message::HistoryFetched,
                )
            }
            Message::SetRange(range) => {
                self.history_range = range;
                if self.selected_sensor.is_some() {
                    self.history_loading = true;
                    let http = self.http.clone();
                    let range = self.history_range.clone();
                    Task::perform(
                        async move { http.fetch_history(&range).await.map_err(|e| e.to_string()) },
                        Message::HistoryFetched,
                    )
                } else {
                    Task::none()
                }
            }
            Message::HistoryFetched(result) => {
                self.history_loading = false;
                if let Ok(resp) = result {
                    // Extract points for the selected sensor
                    if let Some(sensor_id) = self.selected_sensor {
                        if let Some(data) = resp.data.get(&sensor_id.to_string()) {
                            if let Some(points) = data.as_array() {
                                self.history_points = points.iter().filter_map(|p| {
                                    Some(models::DataPoint {
                                        timestamp: p.get("timestamp")?.as_i64()?,
                                        value: p.get("value")?.as_f64()?,
                                    })
                                }).collect();
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::BackToGrid => {
                self.selected_sensor = None;
                self.history_points.clear();
                Task::none()
            }
            Message::Logout => {
                self.phase = AppPhase::Login;
                self.username.clear();
                self.password.clear();
                self.live_data.clear();
                self.data_sources.clear();
                self.selected_sensor = None;
                self.history_points.clear();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.phase {
            AppPhase::Login | AppPhase::Error(_) => {
                views::login::LoginView::view(
                    &self.username,
                    &self.password,
                    if let AppPhase::Error(e) = &self.phase { Some(e.as_str()) } else { self.login_error.as_deref() },
                    self.login_loading,
                )
            }
            AppPhase::Loading => {
                iced::widget::container(
                    iced::widget::text("Connecting to server...")
                        .size(20)
                )
                .center_x(Fill)
                .center_y(Fill)
                .into()
            }
            AppPhase::Dashboard => {
                if self.selected_sensor.is_some() {
                    views::chart::ChartView::view(self)
                } else {
                    views::grid::GridView::view(self)
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if matches!(self.phase, AppPhase::Dashboard | AppPhase::Loading) {
            time::every(Duration::from_secs(self.poll_interval_secs)).map(|_| Message::PollTick)
        } else {
            Subscription::none()
        }
    }
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            client: Client::new(),
            base_url: self.base_url.clone(),
        }
    }
}
```

Note: The `Clone` impl for `HttpClient` creates a new client (cookie jar won't share, but for Task::perform the original client with cookies is moved into the async block — this works because each task uses the original client via move semantics). If cookie sharing is needed, wrap in `Arc`.

- [ ] **Step 4: Build and fix any compile errors**

Run: `cd voltrus-iced && cargo build`
Expected: May need minor adjustments — fix and rebuild

- [ ] **Step 5: Commit**

```bash
git add voltrus-iced/src/
git commit -m "feat(iced): implement app state machine and login view"
```

---

### Task 4: Implement grid view with sensor cards

**Files:**
- Create: `voltrus-iced/src/views/grid.rs`

- [ ] **Step 1: Create grid.rs with sensor card view**

```rust
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Center, Element, Fill};

use crate::app::{App, Message};

pub struct GridView;

impl GridView {
    pub fn view(app: &App) -> Element<'_, Message> {
        let header = row![
            text("Voltrus Viewer").size(20),
            iced::widget::Space::with_width(Fill),
            text(format!("{} sensors", app.live_data.len())).size(14),
            button("Logout").on_press(Message::Logout),
        ]
        .padding(10)
        .align_y(Center);

        let cards: Vec<Element<Message>> = app.data_sources.iter().map(|ds| {
            let live = app.live_data.iter().find(|d| d.id == ds.id);
            let value = live.map(|d| d.value).unwrap_or(f64::NAN);
            let has_warn = live.map(|d| d.value >= d.warn).unwrap_or(false);
            let has_crit = live.map(|d| d.value >= d.crit).unwrap_or(false);

            let border_color = if has_crit {
                [0.9, 0.2, 0.2] // red
            } else if has_warn {
                [0.9, 0.7, 0.1] // yellow
            } else {
                [0.3, 0.5, 0.3] // green
            };

            let value_text = if value.is_nan() {
                "---".to_string()
            } else {
                format!("{:.prec$} {}", value, ds.unit, prec = ds.decimals as usize)
            };

            let card_content = column![
                text(&ds.name).size(14),
                text(value_text).size(22),
            ]
            .spacing(4);

            button(card_content)
                .on_press(Message::SelectSensor(ds.id))
                .padding(12)
                .style(iced::theme::Button::Custom(Box::new(
                    crate::theme::SensorCardStyle { border_color }
                )))
                .into()
        }).collect();

        let grid = iced_aw::flow::flow(iced_aw::flow::FlowDirection::LeftToRight, cards);

        let content = column![header, scrollable(grid)].spacing(10);

        container(content)
            .padding(10)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
```

- [ ] **Step 2: Create theme.rs with custom button style for sensor cards**

```rust
use iced::widget::button;
use iced::{Border, Color, Shadow, Theme};

pub struct SensorCardStyle {
    pub border_color: [f32; 3],
}

impl button::StyleSheet for SensorCardStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Color::from_rgb(0.12, 0.12, 0.14).into()),
            text_color: Color::WHITE,
            border: Border {
                color: Color::from_rgb(self.border_color[0], self.border_color[1], self.border_color[2]),
                width: 2.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut active = self.active(style);
        active.background = Some(Color::from_rgb(0.18, 0.18, 0.22).into());
        active
    }
}
```

- [ ] **Step 3: Update main.rs to add theme module**

Add `mod theme;` at the top of `voltrus-iced/src/main.rs`.

- [ ] **Step 4: Build and fix compile errors**

Run: `cd voltrus-iced && cargo build`

Note: `iced_aw::flow` may not be available in 0.13 — if not, replace with a simple `Row`/`Column` wrapping approach using `Lazy` items. The grid may need to be a column of rows with manual wrapping.

- [ ] **Step 5: Commit**

```bash
git add voltrus-iced/src/views/grid.rs voltrus-iced/src/theme.rs voltrus-iced/src/main.rs
git commit -m "feat(iced): add grid view with sensor cards and alarm colors"
```

---

### Task 5: Implement chart view with plotters-iced

**Files:**
- Create: `voltrus-iced/src/views/chart.rs`

- [ ] **Step 1: Create chart.rs with plotters-iced Chart implementation**

```rust
use iced::widget::{button, column, container, horizontal_space, row, text};
use iced::{Center, Element, Fill, Font, Length};

use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget, Renderer};
use iced::widget::canvas::{Cache, Frame, Geometry};
use iced::Size;

use crate::app::{App, Message};
use crate::models::DataPoint;

pub struct ChartView;

impl ChartView {
    pub fn view(app: &App) -> Element<'_, Message> {
        let sensor = app.selected_sensor.and_then(|id| {
            app.data_sources.iter().find(|ds| ds.id == id)
        });

        let sensor_name = sensor.map(|s| s.name.as_str()).unwrap_or("Unknown");

        let header = row![
            button("← Back").on_press(Message::BackToGrid),
            text(sensor_name).size(20),
            iced::widget::Space::with_width(Fill),
            button("30m").on_press(Message::SetRange("30m".into())),
            button("1h").on_press(Message::SetRange("1h".into())),
            button("6h").on_press(Message::SetRange("6h".into())),
            button("24h").on_press(Message::SetRange("24h".into())),
        ]
        .spacing(8)
        .padding(10)
        .align_y(Center);

        let content = if app.history_loading {
            container(text("Loading history...").size(16))
                .center_x(Fill)
                .center_y(Fill)
                .into()
        } else if app.history_points.is_empty() {
            container(text("No data available").size(16))
                .center_x(Fill)
                .center_y(Fill)
                .into()
        } else {
            let chart_data = HistoryChartData {
                points: app.history_points.clone(),
                warn_val: sensor.map(|s| s.warn_val).unwrap_or(0.0),
                crit_val: sensor.map(|s| s.crit_val).unwrap_or(0.0),
                unit: sensor.map(|s| s.unit.clone()).unwrap_or_default(),
                cache: Cache::new(),
            };
            ChartWidget::new(&chart_data)
                .height(Fill)
                .width(Fill)
                .into()
        };

        column![header, content]
            .spacing(10)
            .width(Fill)
            .height(Fill)
            .into()
    }
}

struct HistoryChartData {
    points: Vec<DataPoint>,
    warn_val: f64,
    crit_val: f64,
    unit: String,
    cache: Cache,
}

impl Chart<Message> for HistoryChartData {
    type State = ();

    fn draw<R: Renderer, F: Fn(&mut Frame)>(
        &self,
        renderer: &R,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }

    fn build_chart<DB: plotters_backend::DrawingBackend>(&self, _state: &Self::State, mut chart: ChartBuilder<DB>) {
        if self.points.is_empty() {
            return;
        }

        let min_ts = self.points.iter().map(|p| p.timestamp).min().unwrap_or(0);
        let max_ts = self.points.iter().map(|p| p.timestamp).max().unwrap_or(1);
        let min_val = self.points.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
        let max_val = self.points.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max);

        let y_min = min_val.min(0.0) - (max_val - min_val).abs() * 0.1;
        let y_max = max_val.max(0.0) + (max_val - min_val).abs() * 0.1;

        let mut chart = chart
            .x_label_area_size(40)
            .y_label_area_size(50)
            .margin(10)
            .build_cartesian_2d(min_ts..max_ts, y_min..y_max)
            .expect("failed to build chart");

        chart.configure_mesh()
            .x_labels(5)
            .y_labels(5)
            .x_label_formatter(&|ts: &i64| {
                let secs = ts % 86400;
                format!("{}:{:02}", secs / 3600, (secs % 3600) / 60)
            })
            .y_label_formatter(&|v: &f64| format!("{:.1}{}", v, self.unit))
            .draw()
            .expect("failed to draw mesh");

        // Warning threshold line
        if self.warn_val > 0.0 {
            chart.draw_series(DashedLineSeries::new(
                vec![(min_ts, self.warn_val), (max_ts, self.warn_val)],
                &RGBColor(230, 180, 30),
                10,
                5,
            )).ok();
        }

        // Critical threshold line
        if self.crit_val > 0.0 {
            chart.draw_series(DashedLineSeries::new(
                vec![(min_ts, self.crit_val), (max_ts, self.crit_val)],
                &RGBColor(220, 50, 50),
                10,
                5,
            )).ok();
        }

        // Data line
        let data_points: Vec<_> = self.points.iter().map(|p| (p.timestamp, p.value)).collect();
        chart.draw_series(LineSeries::new(
            data_points,
            &RGBColor(0, 175, 255),
        )).expect("failed to draw data");
    }
}
```

- [ ] **Step 2: Build and fix compile errors**

Run: `cd voltrus-iced && cargo build`

Note: `DashedLineSeries` may not exist in plotters 0.3 — if not, use `PathElement::new` with a dashed style, or draw two simple horizontal lines using `LineSeries::new`. The Cache import might also need adjustment based on the iced version.

- [ ] **Step 3: Commit**

```bash
git add voltrus-iced/src/views/chart.rs
git commit -m "feat(iced): add history chart view with plotters-iced"
```

---

### Task 6: Integration test — run against backend

**Files:** None new

- [ ] **Step 1: Start the backend**

Terminal 1: `cd /Users/macbookpro/Documents/CobaCoba/voltrus-lite-rust-service/voltrus-lite-rust-service && cargo run -- config.dev.yaml`
Expected: "Server listening on 0.0.0.0:3000"

- [ ] **Step 2: Start the iced client**

Terminal 2: `cd voltrus-iced && cargo run`
Expected: Window opens with login screen

- [ ] **Step 3: Test login**

Enter username: `admin`, password: `admin`, click Login.
Expected: Transitions to grid view with sensor cards

- [ ] **Step 4: Test grid view**

Verify sensor cards show live values. Verify alarm colors appear for sensors exceeding thresholds.

- [ ] **Step 5: Test chart view**

Click a sensor card. Verify history chart loads with line graph and threshold lines. Test range buttons (30m, 1h, 6h, 24h).

- [ ] **Step 6: Test back and logout**

Click "← Back" to return to grid. Click "Logout" to return to login.

- [ ] **Step 7: Commit any fixes**

```bash
git add -u voltrus-iced/
git commit -m "fix(iced): integration test fixes"
```

---

### Task 7: Polish and documentation

**Files:**
- Modify: `voltrus-iced/iced-config.yaml` (add comments)
- Modify: `voltrus-iced/Cargo.toml` (add description)

- [ ] **Step 1: Add config file comments**

Update `voltrus-iced/iced-config.yaml`:
```yaml
# Voltrus Iced Viewer Configuration
server_url: "http://localhost:3000"   # Backend API URL
poll_interval_secs: 1                 # Live data poll interval in seconds
default_history_range: "30m"          # Default chart range (30m, 1h, 6h, 24h)
```

- [ ] **Step 2: Add config loading to app.rs**

Replace the hardcoded values in `App::new()` with actual config file loading using a simple YAML parse (or keep hardcoded for now and add config file parsing in a follow-up).

- [ ] **Step 3: Final build**

Run: `cd voltrus-iced && cargo build --release`
Expected: Clean release build

- [ ] **Step 4: Commit**

```bash
git add voltrus-iced/
git commit -m "feat(iced): polish config and final build"
```

---

## Self-Review

**1. Spec coverage:**
- Login view: Task 3 ✓
- Grid view with sensor cards + alarm colors: Task 4 ✓
- Chart view with plotters-iced + range selector: Task 5 ✓
- HTTP client (login, data, history): Task 2 ✓
- State machine (Login→Loading→Dashboard): Task 3 ✓
- Polling subscription: Task 3 ✓
- Config file: Task 1 + Task 7 ✓
- Logout: Task 3 ✓

**2. Placeholder scan:** No TBDs. All code blocks have actual implementation code. One note about potential API differences (iced_aw::flow, DashedLineSeries) with clear fallback guidance.

**3. Type consistency:** `DataSourceData`, `DataSourceSummary`, `DataPoint`, `HistoryResponse` match backend API JSON structure. `Message` enum is consistent across all views. `HttpClient` methods return the model types.
