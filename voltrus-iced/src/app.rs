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

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

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
    pub data_sources: Vec<models::DataSourceSummary>,
    pub live_data: Vec<models::DataSourceData>,
    pub selected_sensor: Option<i64>,
    pub history_points: Vec<models::DataPoint>,
    pub history_range: String,
    pub history_loading: bool,
    server_url: String,
    poll_interval_secs: u64,
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

pub fn new() -> (App, Task<Message>) {
    let server_url = std::env::var("VOLTRUS_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    let app = App {
        phase: AppPhase::Login,
        http: http::HttpClient::new(&server_url),
        username: String::new(),
        password: String::new(),
        login_error: None,
        login_loading: false,
        data_sources: Vec::new(),
        live_data: Vec::new(),
        selected_sensor: None,
        history_points: Vec::new(),
        history_range: "1h".to_string(),
        history_loading: false,
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
        Message::UsernameChanged(value) => {
            state.username = value;
            state.login_error = None;
            Task::none()
        }

        Message::PasswordChanged(value) => {
            state.password = value;
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
            let username = state.username.clone();
            let password = state.password.clone();
            Task::perform(
                async move {
                    client
                        .login(&username, &password)
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::LoginResult,
            )
        }

        Message::LoginResult(Ok(_resp)) => {
            state.login_loading = false;
            state.phase = AppPhase::Loading;
            // Fetch data sources and live data in parallel
            let client_ds = state.http.clone();
            let client_data = state.http.clone();
            Task::batch([
                Task::perform(
                    async move {
                        client_ds
                            .fetch_data_sources()
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::DataSourcesFetched,
                ),
                Task::perform(
                    async move {
                        client_data
                            .fetch_data()
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::DataFetched,
                ),
            ])
        }

        Message::LoginResult(Err(err)) => {
            state.login_loading = false;
            state.login_error = Some(err);
            Task::none()
        }

        Message::PollTick => {
            match state.phase {
                AppPhase::Dashboard | AppPhase::Loading => {
                    let client = state.http.clone();
                    Task::perform(
                        async move {
                            client
                                .fetch_data()
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::DataFetched,
                    )
                }
                _ => Task::none(),
            }
        }

        Message::DataFetched(Ok(data)) => {
            state.live_data = data;
            if matches!(state.phase, AppPhase::Loading) {
                state.phase = AppPhase::Dashboard;
            }
            Task::none()
        }

        Message::DataFetched(Err(err)) => {
            state.phase = AppPhase::Error(err);
            Task::none()
        }

        Message::DataSourcesFetched(Ok(sources)) => {
            state.data_sources = sources;
            Task::none()
        }

        Message::DataSourcesFetched(Err(err)) => {
            // Non-fatal: we can still show live data without source metadata
            eprintln!("Failed to fetch data sources: {err}");
            Task::none()
        }

        Message::SelectSensor(sensor_id) => {
            state.selected_sensor = Some(sensor_id);
            state.history_points.clear();
            state.history_loading = true;
            let client = state.http.clone();
            let range = state.history_range.clone();
            Task::perform(
                async move {
                    client
                        .fetch_history(&range)
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::HistoryFetched,
            )
        }

        Message::SetRange(range) => {
            state.history_range = range;
            if state.selected_sensor.is_some() {
                state.history_points.clear();
                state.history_loading = true;
                let client = state.http.clone();
                let range = state.history_range.clone();
                Task::perform(
                    async move {
                        client
                            .fetch_history(&range)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::HistoryFetched,
                )
            } else {
                Task::none()
            }
        }

        Message::HistoryFetched(Ok(resp)) => {
            state.history_loading = false;
            state.history_points = extract_history_points(&state.data_sources, state.selected_sensor, &resp);
            Task::none()
        }

        Message::HistoryFetched(Err(err)) => {
            state.history_loading = false;
            eprintln!("Failed to fetch history: {err}");
            Task::none()
        }

        Message::BackToGrid => {
            state.selected_sensor = None;
            state.history_points.clear();
            Task::none()
        }

        Message::Logout => {
            state.phase = AppPhase::Login;
            state.username.clear();
            state.password.clear();
            state.login_error = None;
            state.login_loading = false;
            state.data_sources.clear();
            state.live_data.clear();
            state.selected_sensor = None;
            state.history_points.clear();
            state.history_range = "1h".to_string();
            state.history_loading = false;
            // Create a fresh HTTP client (new cookie jar) to drop the session
            state.http = http::HttpClient::new(&state.server_url);
            Task::none()
        }
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

pub fn view(state: &App) -> Element<'_, Message> {
    match &state.phase {
        AppPhase::Login => crate::views::login::view(state),
        AppPhase::Loading => {
            iced::widget::column![
                iced::widget::Space::with_height(100),
                iced::widget::text("Loading...").size(24),
            ]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .into()
        }
        AppPhase::Dashboard => {
            if state.selected_sensor.is_some() {
                crate::views::chart::view(state)
            } else {
                crate::views::grid::view(state)
            }
        }
        AppPhase::Error(msg) => {
            iced::widget::column![
                iced::widget::Space::with_height(100),
                iced::widget::text(format!("Error: {msg}"))
                    .size(18)
                    .color([0.9, 0.2, 0.2]),
                iced::widget::Space::with_height(20),
                iced::widget::button("Back to Login")
                    .on_press(Message::Logout),
            ]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .into()
        }
    }
}

// ---------------------------------------------------------------------------
// Subscription
// ---------------------------------------------------------------------------

pub fn subscription(state: &App) -> Subscription<Message> {
    if matches!(state.phase, AppPhase::Dashboard | AppPhase::Loading) {
        iced::time::every(Duration::from_secs(state.poll_interval_secs)).map(|_instant| Message::PollTick)
    } else {
        Subscription::none()
    }
}

// ---------------------------------------------------------------------------
// History parsing helper
// ---------------------------------------------------------------------------

/// Extract DataPoints from the compact history response format.
///
/// The backend returns:
/// ```json
/// {
///   "sensors": [...],
///   "data": {
///     "sensor_key": {
///       "values": [1.2, 3.4, 5.6],
///       "start": 1700000000000,
///       "interval": 5000,
///       "count": 3
///     }
///   }
/// }
/// ```
fn extract_history_points(
    data_sources: &[models::DataSourceSummary],
    selected_sensor: Option<i64>,
    resp: &models::HistoryResponse,
) -> Vec<models::DataPoint> {
    // Find the sensor key matching the selected sensor ID
    let sensor_key = match selected_sensor {
        Some(id) => data_sources
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.key.as_str()),
        None => return Vec::new(),
    };

    let sensor_key = match sensor_key {
        Some(key) => key,
        None => return Vec::new(),
    };

    // Navigate the compact JSON structure
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
                timestamp: start + (i as i64) * interval,
                value,
            })
        })
        .collect()
}
