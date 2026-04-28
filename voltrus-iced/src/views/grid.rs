use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::app::{App, Message};
use crate::theme::{self, AlarmLevel};

pub fn view(app: &App) -> Element<'_, Message> {
    // --- Header bar ---
    let title = text("Voltrus Viewer")
        .size(18)
        .color(theme::TEXT)
        .font(iced::Font::DEFAULT);

    let sensor_count = text(format!("{} sensors", app.data_sources.len()))
        .size(12)
        .color(theme::TEXT_MUTED)
        .font(iced::Font::DEFAULT);

    let pause_btn = button(
        text(if app.paused { "Resume" } else { "Pause" })
            .size(12)
            .font(iced::Font::DEFAULT),
    )
    .padding([6, 12])
    .on_press(Message::TogglePause)
    .style(theme::header_btn_style());

    let logout_btn = button(
        text("Logout")
            .size(12)
            .font(iced::Font::DEFAULT),
    )
    .padding([6, 12])
    .on_press(Message::Logout)
    .style(theme::header_btn_style());

    let header_right = row![pause_btn, logout_btn]
        .spacing(8)
        .align_y(Alignment::Center);

    let header = row![
        row![title, sensor_count]
            .spacing(12)
            .align_y(Alignment::Center),
        Space::with_width(Length::Fill),
        header_right,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding(Padding::new(12.0).left(20.0).right(20.0));

    // --- Tabs ---
    let overview_tab = button(
        text("Overview")
            .size(12)
            .font(iced::Font::DEFAULT),
    )
    .padding([8, 16])
    .style(theme::tab_button_style(true));

    let tabs = row![overview_tab]
        .spacing(0)
        .padding(Padding::new(0.0).left(20.0).right(20.0).bottom(4.0));

    // --- Sensor grid ---
    let cards: Vec<Element<'_, Message>> = app
        .data_sources
        .chunks(3)
        .flat_map(|chunk| {
            let mut row_cards: Vec<Element<'_, Message>> = chunk
                .iter()
                .map(|source| sensor_card(source, app))
                .collect();
            // Pad last row to keep widths consistent
            while row_cards.len() < 3 {
                row_cards.push(Space::with_width(Length::FillPortion(1)).into());
            }
            let r = row(row_cards)
                .spacing(12)
                .width(Length::Fill)
                .align_y(Alignment::Start);
            vec![r.into()]
        })
        .collect();

    let grid = column(cards)
        .spacing(12)
        .width(Length::Fill)
        .padding(Padding::new(0.0).left(20.0).right(20.0));

    let scrollable = scrollable(grid);

    // --- Status bar ---
    let status_left = row![
        text("Voltrus Lite").size(11).color(theme::CYAN).font(iced::Font::DEFAULT),
        text("by").size(11).color(theme::TEXT_MUTED).font(iced::Font::DEFAULT),
        text("JoyoDigitama").size(11).color(theme::CYAN).font(iced::Font::DEFAULT),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let live_dot = container(Space::new(Length::Fixed(6.0), Length::Fixed(6.0)))
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::OK)),
            border: iced::border::Border {
                color: theme::OK,
                width: 0.0,
                radius: 3.0.into(),
            },
            ..Default::default()
        });

    let status_right = row![
        live_dot,
        text("LIVE").size(11).color(theme::OK).font(theme::MONO),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let status_bar = container(
        row![
            status_left,
            Space::with_width(Length::Fill),
            status_right,
        ]
        .spacing(12)
        .align_y(Alignment::Center)
    )
    .width(Length::Fill)
    .padding(Padding::new(8.0).left(20.0).right(20.0))
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::CARD)),
        border: iced::border::Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let content = column![
        header,
        tabs,
        scrollable,
        status_bar,
    ]
    .spacing(0)
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::BG)),
            ..Default::default()
        })
        .into()
}

fn sensor_card<'a>(
    source: &'a crate::models::DataSourceSummary,
    app: &'a App,
) -> Element<'a, Message> {
    let live = app.live_data.iter().find(|d| d.id == source.id);

    let (value, alarm) = match live {
        Some(data) if !data.value.is_nan() => {
            let alarm = theme::alarm_level(data.value, data.warn, data.crit);
            (data.value, alarm)
        }
        _ => (f64::NAN, AlarmLevel::Offline),
    };

    let decimals = source.decimals.max(0) as usize;
    let value_str = if value.is_nan() {
        "OFFLINE".to_string()
    } else {
        format!("{:.decimals$}", value, decimals = decimals)
    };

    let status_color = alarm.color();
    let status_bg = alarm.bg_alpha();

    // --- Card header row ---
    let status_dot = container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
        .style(move |_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(status_color)),
            border: iced::border::Border {
                color: status_color,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    let name = text(&source.name)
        .size(12)
        .color(theme::TEXT_DIM)
        .font(iced::Font::DEFAULT);

    let source_tag = text(&source.source_type)
        .size(9)
        .color(theme::TEXT_MUTED)
        .font(theme::MONO);

    let source_tag_container = container(source_tag)
        .padding(Padding::new(2.0).left(5.0).right(5.0))
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.443, 0.443, 0.478, 0.10,
            ))),
            border: iced::border::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 3.0.into(),
            },
            ..Default::default()
        });

    let status_badge = text(alarm.label())
        .size(9)
        .color(status_color)
        .font(iced::Font::DEFAULT);

    let status_badge_container = container(status_badge)
        .padding(Padding::new(2.0).left(6.0).right(6.0))
        .style(move |_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(status_bg)),
            border: iced::border::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    let header_row = row![
        row![status_dot, name, source_tag_container]
            .spacing(8)
            .align_y(Alignment::Center),
        Space::with_width(Length::Fill),
        status_badge_container,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(Padding::new(14.0).left(16.0).right(16.0).bottom(6.0));

    // --- Value row ---
    let value_color = if value.is_nan() {
        theme::TEXT_MUTED
    } else {
        status_color
    };

    let value_text = text(value_str)
        .size(26)
        .color(value_color)
        .font(theme::MONO);

    let unit_text = if source.unit.is_empty() {
        None
    } else {
        Some(
            text(&source.unit)
                .size(13)
                .color(theme::TEXT_MUTED)
                .font(iced::Font::DEFAULT),
        )
    };

    let value_row = {
        let mut r = row![value_text];
        if let Some(u) = unit_text {
            r = r.push(Space::with_width(Length::Fixed(4.0)));
            r = r.push(u);
        }
        r.spacing(0)
            .align_y(Alignment::End)
            .padding(Padding::new(0.0).left(16.0).right(16.0).bottom(14.0))
    };

    // --- Mini bar (value relative to crit or a sensible max) ---
    let bar: Option<Element<'_, Message>> = if !value.is_nan() {
        let max = if source.crit_val > 0.0 {
            source.crit_val * 1.2
        } else if source.warn_val > 0.0 {
            source.warn_val * 1.5
        } else {
            value.max(1.0) * 1.2
        };
        let pct = (value / max).clamp(0.0, 1.0);
        let bar_bg: Element<'_, Message> = container(Space::new(Length::Fill, Length::Fixed(4.0)))
            .style(|_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::BORDER)),
                border: iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .into();
        let bar_fill: Element<'_, Message> = container(Space::new(Length::FillPortion((pct * 100.0) as u16), Length::Fixed(4.0)))
            .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(status_color)),
                border: iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .into();
        let bar_row = row![bar_fill, bar_bg]
            .spacing(0)
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .padding(Padding::new(0.0).left(16.0).right(16.0).bottom(16.0));
        Some(bar_row.into())
    } else {
        None
    };

    let mut card_content = column![header_row, value_row];
    if let Some(b) = bar {
        card_content = card_content.push(b);
    }

    button(card_content)
        .width(Length::FillPortion(1))
        .style(theme::sensor_card_style(alarm))
        .on_press(Message::SelectSensor(source.id))
        .into()
}
