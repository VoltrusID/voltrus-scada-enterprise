use std::collections::HashMap;
use std::time::Duration;

use iced::{Element, Subscription, Task};

use crate::http;
use crate::models;

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum AppPhase {
    Login,
    Loading,
    Dashboard,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Grid,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataMode {
    Live,
    History,
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Message {
    // -- Auth --
    UsernameChanged(String),
    PasswordChanged(String),
    Login,
    LoginResult(Result<models::LoginResponse, String>),
    Logout,

    // -- Data --
    PollTick,
    DataFetched(Result<Vec<models::SensorData>, String>),
    DataSourcesFetched(Result<Vec<models::SensorConfig>, String>),
    SseEvent(String),
    InitialHistorySeed(Result<models::HistoryResponse, String>),

    // -- History --
    HistoryFetched(Result<models::HistoryResponse, String>),
    SetRange(String),

    // -- Navigation --
    SelectSensor(i64),
    BackToGrid,
    SetViewMode(ViewMode),
    SetDataMode(DataMode),
    TogglePause,

    // -- Screens --
    ScreensFetched(Result<Vec<models::Screen>, String>),
    SwitchScreen(String),
    CreateScreen,
    CreateScreenConfirm(String),
    ScreenCreated(Result<models::Screen, String>),
    RenameScreen(String, String),
    RenameScreenConfirm(String, String),
    ScreenRenamed(Result<(), String>),
    DeleteScreen(String, String),
    DeleteScreenConfirm(String),
    ScreenDeleted(Result<(), String>),

    // -- Sensor CRUD --
    ToggleEditMode,
    OpenAddSensor,
    CloseAddSensor,
    AddExistingSensor(i64),
    CreateNewSensor,
    SensorCreated(Result<serde_json::Value, String>),
    OpenEditPanel(i64),
    CloseEditPanel,
    EditFieldChanged(EditField, String),
    SaveSensor,
    SensorSaved(Result<(), String>),
    DeleteSensorConfirm(i64),
    SensorDeleted(Result<(), String>),
    RemoveSensorFromScreen(i64),
    SensorRemovedFromScreen(Result<(), String>),

    // -- Edit form state --
    EditTabChanged(EditTab),

    // -- Alarms --
    AlarmsFetched(Result<Vec<models::Alarm>, String>),
    ToggleAlarmPanel,

    // -- Branding --
    BrandingFetched(Result<models::BrandingSettings, String>),

    // -- Add sensor form --
    AddSensorFormChanged(AddSensorField, String),
}

#[derive(Debug, Clone, Copy)]
pub enum EditField {
    Name,
    Key,
    Unit,
    MinVal,
    MaxVal,
    WarnVal,
    CritVal,
    BaseVal,
    Decimals,
    Color,
    ChartType,
    SourceType,
    SourceConfig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditTab {
    General,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddSensorTab {
    Existing,
    New,
}

#[derive(Debug, Clone, Copy)]
pub enum AddSensorField {
    Key,
    Name,
    Unit,
    MinVal,
    MaxVal,
    BaseVal,
    Decimals,
    SourceType,
    Color,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct App {
    pub phase: AppPhase,
    http: http::HttpClient,
    pub username: String,
    pub password: String,
    pub login_error: Option<String>,
    pub login_loading: bool,
    pub role: String,

    // Data
    pub all_sensors: Vec<models::SensorConfig>,
    pub screen_sensors: Vec<models::SensorConfig>,
    pub live_data: HashMap<i64, models::SensorData>,
    pub live_buffers: HashMap<i64, Vec<models::DataPoint>>, // rolling live data per sensor
    pub history_seeded: bool,

    // Screens
    pub screens: Vec<models::Screen>,
    pub active_screen: String,

    // View state
    pub view_mode: ViewMode,
    pub data_mode: DataMode,
    pub selected_sensor: Option<i64>,
    pub history_points: Vec<models::DataPoint>,
    pub history_range: String,
    pub history_loading: bool,
    pub edit_mode: bool,
    pub paused: bool,

    // Edit panel
    pub editing_sensor_id: Option<i64>,
    pub edit_form: Option<models::SensorConfig>,
    pub edit_tab: EditTab,

    // Add sensor
    pub add_sensor_open: bool,
    pub add_sensor_tab: AddSensorTab,
    pub add_sensor_form: AddSensorForm,

    // Alarms
    pub alarms: Vec<models::Alarm>,
    pub alarm_panel_open: bool,

    // Branding
    pub branding: models::BrandingSettings,

    // Rename / delete screen dialogs
    pub rename_screen_dialog: Option<(String, String)>, // (id, current_name)
    pub delete_screen_dialog: Option<(String, String)>, // (id, name)
    pub create_screen_dialog: Option<String>,
    pub delete_sensor_dialog: Option<i64>,

    server_url: String,
    poll_interval_secs: u64,
}

pub struct AddSensorForm {
    pub key: String,
    pub name: String,
    pub unit: String,
    pub min_val: f64,
    pub max_val: f64,
    pub base_val: f64,
    pub decimals: i64,
    pub source_type: String,
    pub color: String,
}

impl Default for AddSensorForm {
    fn default() -> Self {
        Self {
            key: String::new(),
            name: String::new(),
            unit: String::new(),
            min_val: 0.0,
            max_val: 100.0,
            base_val: 50.0,
            decimals: 1,
            source_type: "random_walk".into(),
            color: "#8b5cf6".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

pub fn new() -> (App, Task<Message>) {
    let server_url = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("VOLTRUS_URL").ok())
        .unwrap_or_else(|| "http://localhost:3000".into());
    let app = App {
        phase: AppPhase::Login,
        http: http::HttpClient::new(&server_url),
        username: String::new(),
        password: String::new(),
        login_error: None,
        login_loading: false,
        role: String::new(),
        all_sensors: Vec::new(),
        screen_sensors: Vec::new(),
        live_data: HashMap::new(),
        live_buffers: HashMap::new(),
        history_seeded: false,
        screens: Vec::new(),
        active_screen: "all".into(),
        view_mode: ViewMode::Grid,
        data_mode: DataMode::Live,
        selected_sensor: None,
        history_points: Vec::new(),
        history_range: "1h".to_string(),
        history_loading: false,
        edit_mode: false,
        paused: false,
        editing_sensor_id: None,
        edit_form: None,
        edit_tab: EditTab::General,
        add_sensor_open: false,
        add_sensor_tab: AddSensorTab::Existing,
        add_sensor_form: AddSensorForm::default(),
        alarms: Vec::new(),
        alarm_panel_open: false,
        branding: models::BrandingSettings {
            title: "Voltrus Lite".into(),
            subtitle: "Monitoring".into(),
            logo: String::new(),
        },
        rename_screen_dialog: None,
        delete_screen_dialog: None,
        create_screen_dialog: None,
        delete_sensor_dialog: None,
        server_url,
        poll_interval_secs: 5,
    };
    (app, Task::none())
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

pub fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        // -- Auth --
        Message::UsernameChanged(v) => {
            state.username = v;
            state.login_error = None;
            Task::none()
        }
        Message::PasswordChanged(v) => {
            state.password = v;
            state.login_error = None;
            Task::none()
        }
        Message::Login => {
            if state.username.is_empty() || state.password.is_empty() {
                state.login_error = Some("Username and password are required".into());
                return Task::none();
            }
            state.login_loading = true;
            state.login_error = None;
            let client = state.http.clone();
            let u = state.username.clone();
            let p = state.password.clone();
            Task::perform(
                async move { client.login(&u, &p).await.map_err(|e| e.to_string()) },
                Message::LoginResult,
            )
        }
        Message::LoginResult(Ok(resp)) => {
            state.login_loading = false;
            state.role = resp.role.clone();
            state.phase = AppPhase::Loading;
            let c1 = state.http.clone();
            let c2 = state.http.clone();
            let c3 = state.http.clone();
            let c4 = state.http.clone();
            let c5 = state.http.clone();
            Task::batch([
                Task::perform(
                    async move { c1.fetch_data_sources().await.map_err(|e| e.to_string()) },
                    Message::DataSourcesFetched,
                ),
                Task::perform(
                    async move { c2.fetch_data().await.map_err(|e| e.to_string()) },
                    Message::DataFetched,
                ),
                Task::perform(
                    async move { c3.fetch_screens().await.map_err(|e| e.to_string()) },
                    Message::ScreensFetched,
                ),
                Task::perform(
                    async move { c4.fetch_branding().await.map_err(|e| e.to_string()) },
                    Message::BrandingFetched,
                ),
                Task::perform(
                    async move { c5.fetch_history("30m").await.map_err(|e| e.to_string()) },
                    Message::InitialHistorySeed,
                ),
            ])
        }
        Message::LoginResult(Err(err)) => {
            state.login_loading = false;
            state.login_error = Some(err);
            Task::none()
        }
        Message::Logout => {
            state.phase = AppPhase::Login;
            state.username.clear();
            state.password.clear();
            state.login_error = None;
            state.login_loading = false;
            state.role.clear();
            state.all_sensors.clear();
            state.screen_sensors.clear();
            state.live_data.clear();
            state.live_buffers.clear();
            state.history_seeded = false;
            state.screens.clear();
            state.active_screen = "all".into();
            state.selected_sensor = None;
            state.history_points.clear();
            state.edit_mode = false;
            state.paused = false;
            state.editing_sensor_id = None;
            state.edit_form = None;
            state.alarms.clear();
            state.http = http::HttpClient::new(&state.server_url);
            Task::none()
        }

        // -- Data --
        Message::PollTick => {
            if state.paused || !matches!(state.phase, AppPhase::Dashboard | AppPhase::Loading) {
                return Task::none();
            }
            // Fetch live data + alarms periodically
            let c1 = state.http.clone();
            let fetch_alarms = state.data_mode == DataMode::Live;
            let c2 = state.http.clone();
            let mut tasks = vec![Task::perform(
                async move { c1.fetch_data().await.map_err(|e| e.to_string()) },
                Message::DataFetched,
            )];
            if fetch_alarms {
                tasks.push(Task::perform(
                    async move { c2.fetch_alarms().await.map_err(|e| e.to_string()) },
                    Message::AlarmsFetched,
                ));
            }
            Task::batch(tasks)
        }
        Message::DataFetched(Ok(data)) => {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            for d in &data {
                state.live_data.insert(d.id, d.clone());
                // Append to rolling live buffer (max 180 pts per sensor)
                let buf = state.live_buffers.entry(d.id).or_default();
                buf.push(models::DataPoint { ts: now_ms, value: d.value });
                if buf.len() > 180 {
                    buf.drain(0..buf.len() - 180);
                }
            }
            if matches!(state.phase, AppPhase::Loading) {
                state.phase = AppPhase::Dashboard;
            }
            Task::none()
        }
        Message::DataFetched(Err(err)) => {
            eprintln!("Data fetch error: {err}");
            Task::none()
        }
        Message::DataSourcesFetched(Ok(sources)) => {
            state.all_sensors = sources;
            // Refresh screen sensors if on 'all'
            if state.active_screen == "all" {
                state.screen_sensors = state.all_sensors.clone();
            }
            if matches!(state.phase, AppPhase::Loading) {
                state.phase = AppPhase::Dashboard;
            }
            Task::none()
        }
        Message::DataSourcesFetched(Err(err)) => {
            eprintln!("Failed to fetch data sources: {err}");
            Task::none()
        }
        Message::SseEvent(raw) => {
            // Parse SSE data payload
            #[derive(serde::Deserialize)]
            struct SensorsPayload {
                sensors: Vec<models::SensorData>,
            }
            if let Ok(data) = serde_json::from_str::<Vec<models::SensorData>>(&raw) {
                for d in data {
                    state.live_data.insert(d.id, d);
                }
            } else if let Ok(wrapped) = serde_json::from_str::<SensorsPayload>(&raw) {
                for d in wrapped.sensors {
                    state.live_data.insert(d.id, d);
                }
            } else if let Ok(data) =
                serde_json::from_str::<std::collections::HashMap<String, models::SensorData>>(&raw)
            {
                for (_k, d) in data {
                    state.live_data.insert(d.id, d);
                }
            }
            if matches!(state.phase, AppPhase::Loading) {
                state.phase = AppPhase::Dashboard;
            }
            Task::none()
        }
        Message::InitialHistorySeed(Ok(resp)) => {
            // Seed live_buffers from 30m history so charts start with data
            let data_obj = match resp.data.as_object() {
                Some(obj) => obj,
                None => {
                    state.history_seeded = true;
                    return Task::none();
                }
            };
            for sensor in &resp.sensors {
                if let Some(sensor_data) = data_obj.get(&sensor.key).and_then(|v| v.as_object()) {
                    let values = match sensor_data.get("values").and_then(|v| v.as_array()) {
                        Some(arr) => arr,
                        None => continue,
                    };
                    let start: i64 = sensor_data.get("start").and_then(|v| v.as_i64()).unwrap_or(0);
                    let interval: i64 = sensor_data.get("interval").and_then(|v| v.as_i64()).unwrap_or(60000);
                    let points: Vec<models::DataPoint> = values
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| {
                            Some(models::DataPoint { ts: start + (i as i64) * interval, value: v.as_f64()? })
                        })
                        .collect();
                    // Keep only last 180 points
                    let trimmed = if points.len() > 180 {
                        points[points.len() - 180..].to_vec()
                    } else {
                        points
                    };
                    state.live_buffers.insert(sensor.id, trimmed);
                }
            }
            state.history_seeded = true;
            if matches!(state.phase, AppPhase::Loading) {
                state.phase = AppPhase::Dashboard;
            }
            Task::none()
        }
        Message::InitialHistorySeed(Err(err)) => {
            eprintln!("Initial history seed failed: {err}");
            state.history_seeded = true;
            Task::none()
        }

        // -- History --
        Message::HistoryFetched(Ok(resp)) => {
            state.history_loading = false;
            state.history_points = extract_history_points(
                &state.all_sensors,
                state.selected_sensor,
                &resp,
            );
            Task::none()
        }
        Message::HistoryFetched(Err(err)) => {
            state.history_loading = false;
            eprintln!("History error: {err}");
            Task::none()
        }
        Message::SetRange(range) => {
            state.history_range = range;
            if state.selected_sensor.is_some() {
                state.history_loading = true;
                state.history_points.clear();
                let c = state.http.clone();
                let r = state.history_range.clone();
                Task::perform(
                    async move { c.fetch_history(&r).await.map_err(|e| e.to_string()) },
                    Message::HistoryFetched,
                )
            } else {
                Task::none()
            }
        }

        // -- Navigation --
        Message::SelectSensor(id) => {
            if state.edit_mode {
                // In edit mode, clicking opens the edit panel instead
                return open_edit_panel(state, id);
            }
            state.selected_sensor = Some(id);
            state.view_mode = ViewMode::Detail;
            state.history_points.clear();
            state.history_loading = true;
            let c = state.http.clone();
            let r = state.history_range.clone();
            Task::perform(
                async move { c.fetch_history(&r).await.map_err(|e| e.to_string()) },
                Message::HistoryFetched,
            )
        }
        Message::BackToGrid => {
            state.selected_sensor = None;
            state.view_mode = ViewMode::Grid;
            state.history_points.clear();
            Task::none()
        }
        Message::SetViewMode(mode) => {
            state.view_mode = mode;
            Task::none()
        }
        Message::SetDataMode(mode) => {
            state.data_mode = mode;
            if mode == DataMode::History && state.selected_sensor.is_some() {
                state.history_loading = true;
                let c = state.http.clone();
                let r = state.history_range.clone();
                Task::perform(
                    async move { c.fetch_history(&r).await.map_err(|e| e.to_string()) },
                    Message::HistoryFetched,
                )
            } else {
                Task::none()
            }
        }
        Message::TogglePause => {
            state.paused = !state.paused;
            Task::none()
        }

        // -- Screens --
        Message::ScreensFetched(Ok(screens)) => {
            state.screens = screens;
            Task::none()
        }
        Message::ScreensFetched(Err(err)) => {
            eprintln!("Screens fetch error: {err}");
            Task::none()
        }
        Message::SwitchScreen(id) => {
            state.active_screen = id.clone();
            state.edit_mode = false;
            state.editing_sensor_id = None;
            state.edit_form = None;
            let c = state.http.clone();
            Task::perform(
                async move {
                    c.fetch_screen_data_sources(&id)
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::DataSourcesFetched,
            )
        }
        Message::CreateScreen => {
            state.create_screen_dialog = Some(String::new());
            Task::none()
        }
        Message::CreateScreenConfirm(name) => {
            let c = state.http.clone();
            state.create_screen_dialog = None;
            Task::perform(
                async move { c.create_screen(&name).await.map_err(|e| e.to_string()) },
                Message::ScreenCreated,
            )
        }
        Message::ScreenCreated(Ok(screen)) => {
            state.screens.push(screen);
            Task::none()
        }
        Message::ScreenCreated(Err(err)) => {
            eprintln!("Create screen error: {err}");
            Task::none()
        }
        Message::RenameScreen(id, name) => {
            state.rename_screen_dialog = Some((id, name));
            Task::none()
        }
        Message::RenameScreenConfirm(id, name) => {
            let c = state.http.clone();
            state.rename_screen_dialog = None;
            Task::perform(
                async move { c.rename_screen(&id, &name).await.map_err(|e| e.to_string()) },
                Message::ScreenRenamed,
            )
        }
        Message::ScreenRenamed(Ok(())) => {
            let c = state.http.clone();
            Task::perform(
                async move { c.fetch_screens().await.map_err(|e| e.to_string()) },
                Message::ScreensFetched,
            )
        }
        Message::ScreenRenamed(Err(err)) => {
            eprintln!("Rename screen error: {err}");
            Task::none()
        }
        Message::DeleteScreen(id, name) => {
            state.delete_screen_dialog = Some((id, name));
            Task::none()
        }
        Message::DeleteScreenConfirm(id) => {
            let c = state.http.clone();
            state.delete_screen_dialog = None;
            // Switch to overview before deleting
            if state.active_screen == id {
                state.active_screen = "all".into();
            }
            Task::perform(
                async move { c.delete_screen(&id).await.map_err(|e| e.to_string()) },
                Message::ScreenDeleted,
            )
        }
        Message::ScreenDeleted(Ok(())) => {
            let c = state.http.clone();
            let c2 = state.http.clone();
            let active = state.active_screen.clone();
            Task::batch([
                Task::perform(
                    async move { c.fetch_screens().await.map_err(|e| e.to_string()) },
                    Message::ScreensFetched,
                ),
                Task::perform(
                    async move {
                        c2.fetch_screen_data_sources(&active)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::DataSourcesFetched,
                ),
            ])
        }
        Message::ScreenDeleted(Err(err)) => {
            eprintln!("Delete screen error: {err}");
            Task::none()
        }

        // -- Sensor CRUD --
        Message::ToggleEditMode => {
            state.edit_mode = !state.edit_mode;
            state.editing_sensor_id = None;
            state.edit_form = None;
            Task::none()
        }
        Message::OpenAddSensor => {
            state.add_sensor_open = true;
            state.add_sensor_tab = AddSensorTab::Existing;
            state.add_sensor_form = AddSensorForm::default();
            Task::none()
        }
        Message::CloseAddSensor => {
            state.add_sensor_open = false;
            Task::none()
        }
        Message::AddExistingSensor(sensor_id) => {
            let c = state.http.clone();
            let screen = state.active_screen.clone();
            state.add_sensor_open = false;
            Task::perform(
                async move {
                    c.add_sensor_to_screen(&screen, sensor_id)
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::SensorRemovedFromScreen, // reuse for refresh
            )
        }
        Message::CreateNewSensor => {
            let c = state.http.clone();
            let form = &state.add_sensor_form;
            let sensor = models::SensorConfig {
                id: 0,
                key: form.key.clone(),
                name: form.name.clone(),
                unit: form.unit.clone(),
                color: form.color.clone(),
                min_val: form.min_val,
                max_val: form.max_val,
                warn_val: form.max_val * 0.8,
                crit_val: form.max_val * 0.9,
                base_val: form.base_val,
                sort_order: 0,
                decimals: form.decimals,
                source_type: form.source_type.clone(),
                source_config: "{}".into(),
                device_id: String::new(),
                equipment_id: None,
                chart_type: "sparkline".into(),
            };
            let _screen = state.active_screen.clone();
            state.add_sensor_open = false;
            Task::perform(
                async move { c.create_sensor(&sensor).await.map_err(|e| e.to_string()) },
                move |res| {
                    // After creating, add to screen
                    // For simplicity, just refresh
                    match res {
                        Ok(_) => Message::DataSourcesFetched(
                            Ok(vec!()), // trigger refresh below
                        ),
                        Err(e) => Message::SensorCreated(Err(e)),
                    }
                },
            )
        }
        Message::SensorCreated(Ok(_)) => {
            refresh_sensors(&state.http, &state.active_screen)
        }
        Message::SensorCreated(Err(err)) => {
            eprintln!("Create sensor error: {err}");
            Task::none()
        }
        Message::OpenEditPanel(id) => open_edit_panel(state, id),
        Message::CloseEditPanel => {
            state.editing_sensor_id = None;
            state.edit_form = None;
            Task::none()
        }
        Message::EditFieldChanged(field, value) => {
            if let Some(form) = &mut state.edit_form {
                match field {
                    EditField::Name => form.name = value,
                    EditField::Key => form.key = value,
                    EditField::Unit => form.unit = value,
                    EditField::MinVal => form.min_val = value.parse().unwrap_or(form.min_val),
                    EditField::MaxVal => form.max_val = value.parse().unwrap_or(form.max_val),
                    EditField::WarnVal => form.warn_val = value.parse().unwrap_or(form.warn_val),
                    EditField::CritVal => form.crit_val = value.parse().unwrap_or(form.crit_val),
                    EditField::BaseVal => form.base_val = value.parse().unwrap_or(form.base_val),
                    EditField::Decimals => form.decimals = value.parse().unwrap_or(form.decimals),
                    EditField::Color => form.color = value,
                    EditField::ChartType => form.chart_type = value,
                    EditField::SourceType => form.source_type = value,
                    EditField::SourceConfig => form.source_config = value,
                }
            }
            Task::none()
        }
        Message::EditTabChanged(tab) => {
            state.edit_tab = tab;
            Task::none()
        }
        Message::SaveSensor => {
            if let Some(form) = state.edit_form.take() {
                let c = state.http.clone();
                state.editing_sensor_id = None;
                Task::perform(
                    async move { c.update_sensor(&form).await.map_err(|e| e.to_string()) },
                    Message::SensorSaved,
                )
            } else {
                Task::none()
            }
        }
        Message::SensorSaved(Ok(())) => {
            refresh_sensors(&state.http, &state.active_screen)
        }
        Message::SensorSaved(Err(err)) => {
            eprintln!("Save sensor error: {err}");
            Task::none()
        }
        Message::DeleteSensorConfirm(id) => {
            state.delete_sensor_dialog = Some(id);
            Task::none()
        }
        Message::SensorDeleted(Ok(())) => {
            state.delete_sensor_dialog = None;
            state.editing_sensor_id = None;
            state.edit_form = None;
            refresh_sensors(&state.http, &state.active_screen)
        }
        Message::SensorDeleted(Err(err)) => {
            eprintln!("Delete sensor error: {err}");
            Task::none()
        }
        Message::RemoveSensorFromScreen(sensor_id) => {
            let c = state.http.clone();
            let screen = state.active_screen.clone();
            Task::perform(
                async move {
                    c.remove_sensor_from_screen(&screen, sensor_id)
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::SensorRemovedFromScreen,
            )
        }
        Message::SensorRemovedFromScreen(Ok(())) => {
            refresh_sensors(&state.http, &state.active_screen)
        }
        Message::SensorRemovedFromScreen(Err(err)) => {
            eprintln!("Remove sensor error: {err}");
            Task::none()
        }

        // -- Add sensor form --
        Message::AddSensorFormChanged(field, value) => {
            let f = &mut state.add_sensor_form;
            match field {
                AddSensorField::Key => f.key = value,
                AddSensorField::Name => f.name = value,
                AddSensorField::Unit => f.unit = value,
                AddSensorField::MinVal => f.min_val = value.parse().unwrap_or(f.min_val),
                AddSensorField::MaxVal => f.max_val = value.parse().unwrap_or(f.max_val),
                AddSensorField::BaseVal => f.base_val = value.parse().unwrap_or(f.base_val),
                AddSensorField::Decimals => f.decimals = value.parse().unwrap_or(f.decimals),
                AddSensorField::SourceType => f.source_type = value,
                AddSensorField::Color => f.color = value,
            }
            Task::none()
        }

        // -- Alarms --
        Message::AlarmsFetched(Ok(alarms)) => {
            state.alarms = alarms;
            Task::none()
        }
        Message::AlarmsFetched(Err(_)) => Task::none(),
        Message::ToggleAlarmPanel => {
            state.alarm_panel_open = !state.alarm_panel_open;
            Task::none()
        }

        // -- Branding --
        Message::BrandingFetched(Ok(b)) => {
            state.branding = b;
            Task::none()
        }
        Message::BrandingFetched(Err(_)) => Task::none(),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn open_edit_panel(state: &mut App, id: i64) -> Task<Message> {
    if state.editing_sensor_id == Some(id) {
        state.editing_sensor_id = None;
        state.edit_form = None;
    } else if let Some(sensor) = state.screen_sensors.iter().find(|s| s.id == id).cloned() {
        state.editing_sensor_id = Some(id);
        state.edit_form = Some(sensor);
        state.edit_tab = EditTab::General;
    }
    Task::none()
}

fn refresh_sensors(http: &http::HttpClient, active_screen: &str) -> Task<Message> {
    let c = http.clone();
    let screen = active_screen.to_string();
    Task::perform(
        async move { c.fetch_screen_data_sources(&screen).await.map_err(|e| e.to_string()) },
        Message::DataSourcesFetched,
    )
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

pub fn view(state: &App) -> Element<'_, Message> {
    match &state.phase {
        AppPhase::Login => crate::views::login::view(state),
        AppPhase::Loading => {
            crate::views::loading::view()
        }
        AppPhase::Dashboard => match state.view_mode {
            ViewMode::Grid => crate::views::grid::view(state),
            ViewMode::Detail => crate::views::detail::view(state),
        },
        AppPhase::Error(msg) => {
            crate::views::error_view::view(msg)
        }
    }
}

// ---------------------------------------------------------------------------
// Subscription
// ---------------------------------------------------------------------------

pub fn subscription(state: &App) -> Subscription<Message> {
    if matches!(state.phase, AppPhase::Dashboard | AppPhase::Loading) && !state.paused {
        iced::time::every(Duration::from_secs(state.poll_interval_secs))
            .map(|_| Message::PollTick)
    } else {
        Subscription::none()
    }
}

// ---------------------------------------------------------------------------
// History parsing helper
// ---------------------------------------------------------------------------

fn extract_history_points(
    data_sources: &[models::SensorConfig],
    selected_sensor: Option<i64>,
    resp: &models::HistoryResponse,
) -> Vec<models::DataPoint> {
    let sensor_key = match selected_sensor {
        Some(id) => data_sources.iter().find(|s| s.id == id).map(|s| s.key.as_str()),
        None => return Vec::new(),
    };
    let sensor_key = match sensor_key {
        Some(k) => k,
        None => return Vec::new(),
    };
    let data_obj = match resp.data.as_object() {
        Some(obj) => obj,
        None => return Vec::new(),
    };
    let sensor_data = match data_obj.get(sensor_key).and_then(|v| v.as_object()) {
        Some(obj) => obj,
        None => return Vec::new(),
    };
    let values = match sensor_data.get("values").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };
    let start: i64 = sensor_data
        .get("start")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let interval: i64 = sensor_data
        .get("interval")
        .and_then(|v| v.as_i64())
        .unwrap_or(60000);

    values
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            let value = v.as_f64()?;
            Some(models::DataPoint {
                ts: start + (i as i64) * interval,
                value,
            })
        })
        .collect()
}
