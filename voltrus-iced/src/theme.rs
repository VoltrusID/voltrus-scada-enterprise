use iced::border;
use iced::widget::button::{self, Status};
use iced::{Background, Color};

/// Alarm severity levels for sensor status coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmLevel {
    /// Normal operating range.
    Normal,
    /// Value has crossed the warning threshold.
    Warning,
    /// Value has crossed the critical threshold.
    Critical,
}

/// Determine the alarm level from a sensor value and its thresholds.
pub fn alarm_level(value: f64, warn: f64, crit: f64) -> AlarmLevel {
    if value.is_nan() {
        return AlarmLevel::Normal;
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
    /// Return the border color for this alarm level.
    pub fn border_color(self) -> Color {
        match self {
            AlarmLevel::Normal => Color::from_rgb(0.2, 0.75, 0.3),
            AlarmLevel::Warning => Color::from_rgb(0.9, 0.75, 0.1),
            AlarmLevel::Critical => Color::from_rgb(0.9, 0.2, 0.2),
        }
    }

    /// Return the value text color for this alarm level.
    pub fn text_color(self) -> Color {
        match self {
            AlarmLevel::Normal => Color::from_rgb(0.5, 0.95, 0.6),
            AlarmLevel::Warning => Color::from_rgb(1.0, 0.9, 0.3),
            AlarmLevel::Critical => Color::from_rgb(1.0, 0.4, 0.4),
        }
    }
}

/// Build a sensor card button style function.
///
/// Returns a closure compatible with `button().style(...)` that produces
/// a dark card with a colored border indicating alarm status.
pub fn sensor_card_style(alarm: AlarmLevel) -> impl Fn(&iced::Theme, Status) -> button::Style {
    move |_theme: &iced::Theme, status: Status| {
        let bg = match status {
            Status::Hovered => Color::from_rgb(0.18, 0.18, 0.20),
            _ => Color::from_rgb(0.12, 0.12, 0.14),
        };

        let border_color = alarm.border_color();

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::WHITE,
            border: border::Border {
                color: border_color,
                width: 2.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}
