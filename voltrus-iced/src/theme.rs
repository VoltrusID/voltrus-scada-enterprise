use iced::border;
use iced::widget::button::{self, Status};
use iced::{Background, Color, Font};

// ---------------------------------------------------------------------------
// Web palette (matches client/src/index.css)
// ---------------------------------------------------------------------------

pub const BG: Color = Color::from_rgb(0.035, 0.035, 0.043); // #09090b
pub const CARD: Color = Color::from_rgb(0.067, 0.067, 0.075); // #111113
pub const BORDER: Color = Color::from_rgb(0.153, 0.153, 0.165); // #27272a
#[allow(dead_code)]
pub const BORDER_LIGHT: Color = Color::from_rgb(0.247, 0.247, 0.275); // #3f3f46
pub const TEXT: Color = Color::from_rgb(0.894, 0.894, 0.906); // #e4e4e7
pub const TEXT_DIM: Color = Color::from_rgb(0.631, 0.631, 0.667); // #a1a1aa
pub const TEXT_MUTED: Color = Color::from_rgb(0.443, 0.443, 0.478); // #71717a
pub const WARN: Color = Color::from_rgb(0.961, 0.620, 0.043); // #f59e0b
pub const CRIT: Color = Color::from_rgb(0.937, 0.267, 0.267); // #ef4444
pub const OK: Color = Color::from_rgb(0.133, 0.773, 0.369); // #22c55e
pub const CYAN: Color = OK;
pub const PURPLE: Color = Color::from_rgb(0.654, 0.545, 0.980); // #a78bfa

pub const MONO: Font = Font::MONOSPACE;

// ---------------------------------------------------------------------------
// Alarm severity levels
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmLevel {
    Normal,
    Warning,
    Critical,
    Offline,
}

pub fn alarm_level(value: f64, warn: f64, crit: f64) -> AlarmLevel {
    if value.is_nan() {
        return AlarmLevel::Offline;
    }
    if value >= crit {
        AlarmLevel::Critical
    } else if value >= warn {
        AlarmLevel::Warning
    } else {
        AlarmLevel::Normal
    }
}

impl AlarmLevel {
    pub fn color(self) -> Color {
        match self {
            AlarmLevel::Normal => OK,
            AlarmLevel::Warning => WARN,
            AlarmLevel::Critical => CRIT,
            AlarmLevel::Offline => TEXT_MUTED,
        }
    }

    pub fn bg_alpha(self) -> Color {
        match self {
            AlarmLevel::Normal => Color::from_rgba(0.133, 0.773, 0.369, 0.15),
            AlarmLevel::Warning => Color::from_rgba(0.961, 0.620, 0.043, 0.15),
            AlarmLevel::Critical => Color::from_rgba(0.937, 0.267, 0.267, 0.15),
            AlarmLevel::Offline => Color::from_rgba(0.392, 0.392, 0.392, 0.15),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AlarmLevel::Normal => "Normal",
            AlarmLevel::Warning => "Warning",
            AlarmLevel::Critical => "Critical",
            AlarmLevel::Offline => "Offline",
        }
    }
}

// ---------------------------------------------------------------------------
// Sensor card button style
// ---------------------------------------------------------------------------

pub fn sensor_card_style(alarm: AlarmLevel) -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, status: Status| {
        let bg = match status {
            Status::Hovered => Color::from_rgb(0.10, 0.10, 0.11),
            _ => CARD,
        };

        let border_color = if alarm == AlarmLevel::Normal {
            BORDER
        } else {
            alarm.color()
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT,
            border: border::Border {
                color: border_color,
                width: if alarm == AlarmLevel::Normal {
                    1.0
                } else {
                    1.5
                },
                radius: 10.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Range / tab button styles
// ---------------------------------------------------------------------------

pub fn tab_button_style(active: bool) -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, status: Status| {
        let (bg, text_col, border_col) = if active {
            (
                Color::from_rgba(0.133, 0.773, 0.369, 0.10),
                CYAN,
                CYAN,
            )
        } else {
            let bg = match status {
                Status::Hovered => Color::from_rgb(0.15, 0.15, 0.17),
                _ => Color::TRANSPARENT,
            };
            (bg, TEXT_MUTED, Color::TRANSPARENT)
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: text_col,
            border: border::Border {
                color: border_col,
                width: if active { 2.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Header button style (small grey buttons)
// ---------------------------------------------------------------------------

pub fn header_btn_style() -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, status: Status| {
        let bg = match status {
            Status::Hovered => Color::from_rgb(0.18, 0.18, 0.20),
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT_DIM,
            border: border::Border {
                color: BORDER,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Primary action button (green)
// ---------------------------------------------------------------------------

pub fn primary_btn_style() -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, status: Status| {
        let bg = match status {
            Status::Hovered => Color::from_rgb(0.18, 0.85, 0.42),
            _ => OK,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::BLACK,
            border: border::Border {
                color: OK,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Mode toggle button style
// ---------------------------------------------------------------------------

pub fn mode_toggle_style(
    active: bool,
    active_color: Color,
) -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, _status: Status| {
        if active {
            button::Style {
                background: Some(Background::Color(Color::from_rgba(
                    active_color.r,
                    active_color.g,
                    active_color.b,
                    0.10,
                ))),
                text_color: active_color,
                border: border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        } else {
            button::Style {
                background: Some(Background::Color(CARD)),
                text_color: TEXT,
                border: border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Active button style (outlined with color)
// ---------------------------------------------------------------------------

pub fn active_btn_style(
    color: Color,
) -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, _status: Status| button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color,
        border: border::Border {
            color,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

// ---------------------------------------------------------------------------
// Danger button style (red outlined)
// ---------------------------------------------------------------------------

pub fn danger_btn_style() -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, _status: Status| button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: CRIT,
        border: border::Border {
            color: CRIT,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

// ---------------------------------------------------------------------------
// Remove button style (small red)
// ---------------------------------------------------------------------------

pub fn remove_btn_style() -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, _status: Status| button::Style {
        background: Some(Background::Color(BG)),
        text_color: CRIT,
        border: border::Border {
            color: CRIT,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

// ---------------------------------------------------------------------------
// Icon button style (tiny, no border)
// ---------------------------------------------------------------------------

pub fn icon_btn_style() -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, _status: Status| button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: TEXT_MUTED,
        border: border::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

// ---------------------------------------------------------------------------
// List item style (for sensor list in add dialog)
// ---------------------------------------------------------------------------

pub fn list_item_style() -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, status: Status| {
        let bg = match status {
            Status::Hovered => Color::from_rgb(0.10, 0.10, 0.12),
            _ => BG,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT,
            border: border::Border {
                color: BORDER,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Text input style
// ---------------------------------------------------------------------------

pub fn text_input_style(
    _theme: &iced::Theme,
    _status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: Background::Color(BG),
        border: border::Border {
            color: BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT,
        selection: iced::Color::from_rgb(0.2, 0.4, 0.65),
    }
}
