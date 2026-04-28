use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::app::{App, Message};
use crate::theme::{self, AlarmLevel};

pub fn view(app: &App) -> Element<'_, Message> {
    // --- Header ---
    let sensor_count = app.data_sources.len();
    let header = row![
        text("Voltrus Viewer")
            .size(22)
            .width(Length::Fill),
        text(format!("{} sensors", sensor_count))
            .size(14)
            .color([0.6, 0.6, 0.6]),
        button(text("Logout").size(14))
            .padding([6, 16])
            .on_press(Message::Logout),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center)
    .padding([12, 16]);

    // --- Build sensor cards ---
    let cards: Vec<Element<'_, Message>> = app
        .data_sources
        .iter()
        .map(|source| sensor_card(source, app))
        .collect();

    // --- Scrollable grid ---
    let cards_column = column(cards).spacing(8).padding([0, 16]);

    let scrollable = scrollable(cards_column);

    let content = column![
        header,
        Space::with_height(8),
        scrollable,
    ]
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn sensor_card<'a>(
    source: &'a crate::models::DataSourceSummary,
    app: &'a App,
) -> Element<'a, Message> {
    // Find matching live data
    let live = app.live_data.iter().find(|d| d.id == source.id);

    let (value_str, alarm) = match live {
        Some(data) if !data.value.is_nan() => {
            let decimals = source.decimals.max(0) as usize;
            let formatted = format!("{:.decimals$}", data.value, decimals = decimals);
            let alarm = theme::alarm_level(data.value, data.warn, data.crit);
            (formatted, alarm)
        }
        _ => ("---".to_string(), AlarmLevel::Normal),
    };

    let unit = if source.unit.is_empty() {
        String::new()
    } else {
        format!(" {}", source.unit)
    };

    let card_content = column![
        text(&source.name)
            .size(14)
            .color([0.85, 0.85, 0.85])
            .width(Length::Fill),
        Space::with_height(4),
        text(format!("{}{}", value_str, unit))
            .size(26)
            .color(alarm.text_color()),
    ]
    .spacing(0)
    .padding([12.0, 16.0]);

    button(card_content)
        .width(Length::Fill)
        .style(theme::sensor_card_style(alarm))
        .on_press(Message::SelectSensor(source.id))
        .into()
}
