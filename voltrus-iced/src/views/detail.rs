use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use plotters_iced::ChartWidget;

use crate::app::{App, DataMode, Message};
use crate::models;
use crate::theme;

const RANGES: &[&str] = &["30m", "1h", "6h", "24h"];

pub fn view(app: &App) -> Element<'_, Message> {
    let sensor = app
        .all_sensors
        .iter()
        .find(|s| Some(s.id) == app.selected_sensor);

    let sensor = match sensor {
        Some(s) => s,
        None => {
            return column![
                Space::with_height(60),
                text("Sensor not found").size(16).color(theme::CRIT),
                Space::with_height(12),
                button(text("Back").size(12))
                    .padding([6, 12])
                    .on_press(Message::BackToGrid)
                    .style(theme::header_btn_style()),
            ]
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .into();
        }
    };

    let live = app.live_data.get(&sensor.id);
    let alarm = theme::alarm_level(
        live.map(|d| d.value).unwrap_or(f64::NAN),
        sensor.warn_val,
        sensor.crit_val,
    );
    let is_offline = matches!(alarm, theme::AlarmLevel::Offline);
    let status_color = alarm.color();
    let status_bg = alarm.bg_alpha();
    let status_label = alarm.label();

    let cur_v = if is_offline {
        None
    } else {
        live.map(|d| d.value).or(Some(sensor.base_val))
    };
    let decimals = sensor.decimals.max(0) as usize;

    // Pick chart data source based on mode
    let chart_pts: Vec<models::DataPoint> = if matches!(app.data_mode, DataMode::Live) {
        match app.live_buffers.get(&sensor.id) {
            Some(b) => b.clone(),
            None => Vec::new(),
        }
    } else {
        app.history_points.clone()
    };

    // Pre-format all stat values to own them
    let values: Vec<f64> = chart_pts
        .iter()
        .map(|p| p.value)
        .filter(|v| v.is_finite())
        .collect();
    let min_v = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_v = values
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let avg_v = if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    };

    let cur_str = match cur_v {
        Some(v) => format!("{:.decimals$}{}", v, sensor.unit, decimals = decimals),
        None => "OFFLINE".into(),
    };
    let min_str = if values.is_empty() {
        "\u{2014}".into()
    } else {
        format!("{:.decimals$}", min_v, decimals = decimals)
    };
    let max_str = if values.is_empty() {
        "\u{2014}".into()
    } else {
        format!("{:.decimals$}", max_v, decimals = decimals)
    };
    let avg_str = if values.is_empty() {
        "\u{2014}".into()
    } else {
        format!("{:.decimals$}", avg_v, decimals = decimals)
    };
    let pts_str = format!("{}", chart_pts.len());

    // ── Header row ──
    let back_btn = button(
        row![text("< ").size(12), text("Back").size(12)]
            .spacing(2)
            .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .on_press(Message::BackToGrid)
    .style(theme::header_btn_style());

    let title = text(&sensor.name).size(20).color(theme::TEXT);

    let key_label = text(format!("({})", sensor.key))
        .size(12)
        .color(theme::TEXT_MUTED);

    // Live/History toggle
    let live_btn = button(text("Live").size(12))
        .padding([4, 10])
        .style(theme::mode_toggle_style(
            matches!(app.data_mode, DataMode::Live),
            theme::OK,
        ));
    let live_btn = if !matches!(app.data_mode, DataMode::Live) {
        live_btn.on_press(Message::SetDataMode(DataMode::Live))
    } else {
        live_btn
    };

    let hist_btn = button(text("History").size(12))
        .padding([4, 10])
        .style(theme::mode_toggle_style(
            matches!(app.data_mode, DataMode::History),
            theme::WARN,
        ));
    let hist_btn = if !matches!(app.data_mode, DataMode::History) {
        hist_btn.on_press(Message::SetDataMode(DataMode::History))
    } else {
        hist_btn
    };

    let mode_toggle = container(row![live_btn, hist_btn].spacing(0))
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            border: iced::border::Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

    // Range selector (only in history mode)
    let mut range_row = row![].spacing(0);
    if matches!(app.data_mode, DataMode::History) {
        for range in RANGES {
            let active = app.history_range == *range;
            let btn = button(text(*range).size(11))
                .padding([4, 10])
                .on_press(Message::SetRange(range.to_string()))
                .style(theme::tab_button_style(active));
            range_row = range_row.push(btn);
        }
    }

    // Status badge
    let badge = text(status_label).size(12).color(status_color);
    let badge_box = container(badge)
        .padding([4, 12])
        .style(move |_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(status_bg)),
            border: iced::border::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

    let source_label = text(format!("Source: {}", sensor.source_type))
        .size(12)
        .color(theme::TEXT_MUTED);

    let header_right = row![mode_toggle, range_row, badge_box, source_label]
        .spacing(8)
        .align_y(Alignment::Center);

    let header = row![
        row![back_btn, title, key_label]
            .spacing(12)
            .align_y(Alignment::Center),
        Space::with_width(Length::Fill),
        header_right,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding(Padding::new(12.0).left(20.0).right(20.0));

    // ── Stats row ──
    let cur_card = stat_card("Current".into(), cur_str, Some(status_color));
    let min_card = stat_card("Min".into(), min_str, None);
    let max_card = stat_card("Max".into(), max_str, None);
    let avg_card = stat_card("Avg".into(), avg_str, None);
    let pts_card = stat_card("Points".into(), pts_str, None);

    let warn_str = format!("{:.1}{}", sensor.warn_val, sensor.unit);
    let crit_str = format!("{:.1}{}", sensor.crit_val, sensor.unit);
    let range_str = format!("{}\u{2013}{}", sensor.min_val, sensor.max_val);

    let threshold_card = container(
        column![
            row![
                text("Warn").size(11).color(theme::TEXT_MUTED),
                Space::with_width(Length::Fill),
                text(warn_str).size(11).color(theme::WARN).font(theme::MONO),
            ],
            row![
                text("Crit").size(11).color(theme::TEXT_MUTED),
                Space::with_width(Length::Fill),
                text(crit_str).size(11).color(theme::CRIT).font(theme::MONO),
            ],
            row![
                text("Range").size(11).color(theme::TEXT_MUTED),
                Space::with_width(Length::Fill),
                text(range_str).size(11).color(theme::TEXT).font(theme::MONO),
            ],
        ]
        .spacing(4),
    )
    .padding([10, 14])
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::CARD)),
        border: iced::border::Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    let stats_row = row![cur_card, min_card, max_card, avg_card, pts_card, threshold_card,]
        .spacing(8)
        .padding(Padding::new(0.0).left(20.0).right(20.0));

    // ── Chart area ──
    let chart_content: Element<'_, Message> = if app.history_loading {
        container(
            column![
                Space::with_height(60),
                text("Loading...").size(14).color(theme::TEXT_MUTED),
            ]
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else if chart_pts.is_empty() {
        container(
            column![
                Space::with_height(60),
                text("No data").size(14).color(theme::TEXT_MUTED),
            ]
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        let chart = crate::views::chart::HistoryChart::new(
            chart_pts,
            sensor.warn_val,
            sensor.crit_val,
            sensor.unit.clone(),
        );
        container(ChartWidget::new(chart).width(Length::Fill).height(Length::Fill))
            .padding(4)
            .into()
    };

    let chart_container = container(chart_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(0.0).left(20.0).right(20.0))
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::CARD)),
            border: iced::border::Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        });

    column![header, stats_row, Space::with_height(8), chart_container,]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(0.0).bottom(12.0))
        .into()
}

fn stat_card(label: String, value: String, color: Option<Color>) -> Element<'static, Message> {
    let value_color = color.unwrap_or(theme::TEXT);
    container(
        column![
            text(label).size(10).color(theme::TEXT_MUTED).width(Length::Fill),
            text(value).size(16).color(value_color).font(theme::MONO),
        ]
        .spacing(2),
    )
    .padding([10, 14])
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::CARD)),
        border: iced::border::Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}
