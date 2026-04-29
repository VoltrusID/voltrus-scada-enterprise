use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::app::{App, Message};
use crate::models;
use crate::theme::{self, AlarmLevel};

// ---------------------------------------------------------------------------
// Grid view (main dashboard)
// ---------------------------------------------------------------------------

pub fn view(app: &App) -> Element<'_, Message> {
    let content = column![
        header(app),
        screen_tabs(app),
        sensor_grid(app),
        status_bar(),
    ]
    .spacing(0)
    .height(Length::Fill);

    let base = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::BG)),
            ..Default::default()
        });

    // Overlay edit panel if open
    if app.editing_sensor_id.is_some() && app.edit_form.is_some() {
        let edit = crate::views::edit_panel::view(app);
        iced::widget::stack![base, edit].into()
    } else if app.add_sensor_open {
        let add = crate::views::add_sensor::view(app);
        iced::widget::stack![base, add].into()
    } else if let Some(sensor_id) = app.delete_sensor_dialog {
        let dialog = delete_confirm(sensor_id);
        iced::widget::stack![base, dialog].into()
    } else if let Some((id, name)) = &app.rename_screen_dialog {
        let dialog = rename_screen_dialog(id, name);
        iced::widget::stack![base, dialog].into()
    } else if let Some((id, name)) = &app.delete_screen_dialog {
        let dialog = delete_screen_dialog(id, name);
        iced::widget::stack![base, dialog].into()
    } else if let Some(name) = &app.create_screen_dialog {
        let dialog = create_screen_dialog(name);
        iced::widget::stack![base, dialog].into()
    } else {
        base.into()
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn header(app: &App) -> Element<'_, Message> {
    // Left: branding
    let brand = row![
        text(&app.branding.title)
            .size(20)
            .color(theme::TEXT)
            .font(iced::Font::DEFAULT),
        text(&app.branding.subtitle)
            .size(14)
            .color(theme::TEXT_MUTED)
            .font(iced::Font::DEFAULT),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // Right: controls
    let alarm_count = app
        .alarms
        .iter()
        .filter(|a| a.resolved_at.is_none())
        .count();

    let alarm_btn = button(
        row![
            text("\u{1F514}").size(12),
            if alarm_count > 0 {
                text(format!("{}", alarm_count))
                    .size(10)
                    .color(theme::CRIT)
                    .font(theme::MONO)
            } else {
                text("").size(10)
            },
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .padding([6, 10])
    .on_press(Message::ToggleAlarmPanel)
    .style(theme::header_btn_style());

    // Live/History toggle
    let live_btn = button(text("Live").size(12))
        .padding([6, 12])
        .style(theme::mode_toggle_style(
            matches!(app.data_mode, crate::app::DataMode::Live),
            theme::OK,
        ));
    let live_btn = if matches!(app.data_mode, crate::app::DataMode::Live) {
        live_btn
    } else {
        live_btn.on_press(Message::SetDataMode(crate::app::DataMode::Live))
    };

    let hist_btn = button(text("History").size(12))
        .padding([6, 12])
        .style(theme::mode_toggle_style(
            matches!(app.data_mode, crate::app::DataMode::History),
            theme::WARN,
        ));
    let hist_btn = if matches!(app.data_mode, crate::app::DataMode::History) {
        hist_btn
    } else {
        hist_btn.on_press(Message::SetDataMode(crate::app::DataMode::History))
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

    let pause_btn = button(
        text(if app.paused { "Resume" } else { "Pause" }).size(12),
    )
    .padding([6, 12])
    .on_press(Message::TogglePause)
    .style(theme::header_btn_style());

    let data_btn = button(text("Data").size(12))
        .padding([6, 12])
        .on_press(Message::OpenAddSensor)
        .style(theme::header_btn_style());

    let logout_btn = button(text("Logout").size(12))
        .padding([6, 12])
        .on_press(Message::Logout)
        .style(theme::header_btn_style());

    let right = row![alarm_btn, mode_toggle, pause_btn, data_btn, logout_btn]
        .spacing(8)
        .align_y(Alignment::Center);

    row![brand, Space::with_width(Length::Fill), right,]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(Padding::new(12.0).left(20.0).right(20.0))
        .into()
}

// ---------------------------------------------------------------------------
// Screen Tabs
// ---------------------------------------------------------------------------

fn screen_tabs(app: &App) -> Element<'_, Message> {
    let mut tabs_row = row![].spacing(0);

    // Overview tab
    let overview_active = app.active_screen == "all";
    let overview_btn = button(text("Overview").size(12))
        .padding([8, 16])
        .style(theme::tab_button_style(overview_active));
    let overview_btn = if !overview_active {
        overview_btn.on_press(Message::SwitchScreen("all".into()))
    } else {
        overview_btn
    };
    tabs_row = tabs_row.push(overview_btn);

    // Screen tabs
    for screen in &app.screens {
        let is_active = app.active_screen == screen.id;
        let tab_btn = button(text(&screen.name).size(12))
            .padding([8, 16])
            .style(theme::tab_button_style(is_active));
        let tab_btn = if !is_active {
            tab_btn.on_press(Message::SwitchScreen(screen.id.clone()))
        } else {
            tab_btn
        };

        if app.edit_mode {
            tabs_row = tabs_row.push(
                row![tab_btn]
                    .spacing(0)
                    .push(
                        button(text("x").size(9).color(theme::CRIT))
                            .padding([2, 4])
                            .on_press(Message::DeleteScreen(
                                screen.id.clone(),
                                screen.name.clone(),
                            ))
                            .style(theme::icon_btn_style()),
                    )
                    .push(
                        button(text("~").size(9).color(theme::TEXT_MUTED))
                            .padding([2, 4])
                            .on_press(Message::RenameScreen(
                                screen.id.clone(),
                                screen.name.clone(),
                            ))
                            .style(theme::icon_btn_style()),
                    ),
            );
        } else {
            tabs_row = tabs_row.push(tab_btn);
        }
    }

    // Add screen button (edit mode)
    if app.edit_mode {
        tabs_row = tabs_row.push(
            button(text("+").size(12).color(theme::OK))
                .padding([8, 12])
                .on_press(Message::CreateScreen)
                .style(theme::tab_button_style(false)),
        );
    }

    // Edit mode toggle (right side)
    let is_admin = app.role == "admin";
    let edit_label = if app.edit_mode { "Done" } else { "Edit" };
    let right_controls = if is_admin {
        let mut r = row![].spacing(8);
        let edit_btn = button(text(edit_label).size(12))
            .padding([6, 12])
            .style(theme::header_btn_style())
            .on_press(Message::ToggleEditMode);
        r = r.push(edit_btn);

        if app.edit_mode && app.active_screen != "all" {
            r = r.push(
                button(text("+ Source").size(12).color(theme::OK))
                    .padding([6, 12])
                    .on_press(Message::OpenAddSensor)
                    .style(theme::active_btn_style(theme::OK)),
            );
        }
        Some(r)
    } else {
        None
    };

    let mut tab_bar = row![].spacing(0).push(tabs_row);

    if let Some(right) = right_controls {
        tab_bar = tab_bar.push(Space::with_width(Length::Fill)).push(right);
    }

    container(tab_bar)
        .width(Length::Fill)
        .padding(Padding::new(0.0).left(20.0).right(20.0))
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            border: iced::border::Border {
                color: theme::BORDER,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// ---------------------------------------------------------------------------
// Sensor Grid
// ---------------------------------------------------------------------------

fn sensor_grid(app: &App) -> Element<'_, Message> {
    if app.screen_sensors.is_empty() {
        return container(
            column![
                Space::with_height(60),
                text("No sensors on this screen").size(14).color(theme::TEXT_MUTED),
            ]
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    let cards: Vec<Element<'_, Message>> = app
        .screen_sensors
        .chunks(3)
        .flat_map(|chunk| {
            let mut row_cards: Vec<Element<'_, Message>> = chunk
                .iter()
                .map(|s| sensor_card(s, app))
                .collect();
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
        .padding(Padding::new(0.0).left(20.0).right(20.0).top(4.0));

    scrollable(grid)
        .height(Length::Fill)
        .into()
}

// ---------------------------------------------------------------------------
// Sensor Card
// ---------------------------------------------------------------------------

fn sensor_card<'a>(
    sensor: &'a models::SensorConfig,
    app: &'a App,
) -> Element<'a, Message> {
    let live = app.live_data.get(&sensor.id);

    let (value, _status_str, alarm) = match live {
        Some(data) if !data.value.is_nan() => {
            let alarm = theme::alarm_level(data.value, sensor.warn_val, sensor.crit_val);
            (data.value, data.status.clone(), alarm)
        }
        _ => (f64::NAN, "offline".into(), AlarmLevel::Offline),
    };

    let decimals = sensor.decimals.max(0) as usize;
    let value_str = if value.is_nan() {
        "OFFLINE".to_string()
    } else {
        format!("{:.decimals$}", value, decimals = decimals)
    };

    let status_color = alarm.color();
    let status_bg = alarm.bg_alpha();

    // Status dot
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

    // Name
    let name = text(&sensor.name)
        .size(13)
        .color(theme::TEXT_DIM)
        .font(iced::Font::DEFAULT);

    // Source tag
    let source_tag = text(&sensor.source_type)
        .size(9)
        .color(theme::TEXT_MUTED)
        .font(theme::MONO);
    let source_tag_box = container(source_tag)
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

    // Status badge
    let badge = text(alarm.label())
        .size(9)
        .color(status_color);
    let badge_box = container(badge)
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

    // Remove button in edit mode
    let mut header_right = row![].spacing(6).align_y(Alignment::Center);
    if app.edit_mode && app.active_screen != "all" {
        header_right = header_right.push(
            button(text("Remove").size(10).color(theme::CRIT))
                .padding([2, 6])
                .on_press(Message::RemoveSensorFromScreen(sensor.id))
                .style(theme::remove_btn_style()),
        );
    }
    header_right = header_right.push(badge_box);

    // Card header
    let card_header = row![
        row![status_dot, name, source_tag_box]
            .spacing(8)
            .align_y(Alignment::Center),
        Space::with_width(Length::Fill),
        header_right,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(Padding::new(14.0).left(16.0).right(16.0).bottom(6.0));

    // Value
    let value_color = if value.is_nan() {
        theme::TEXT_MUTED
    } else {
        status_color
    };
    let value_text = text(value_str)
        .size(22)
        .color(value_color)
        .font(theme::MONO);

    let unit_text = if sensor.unit.is_empty() {
        None
    } else {
        Some(text(&sensor.unit).size(13).color(theme::TEXT_MUTED))
    };

    let mut value_row = row![value_text].spacing(4).align_y(Alignment::End);
    if let Some(u) = unit_text {
        value_row = value_row.push(u);
    }
    let value_row = value_row
        .padding(Padding::new(0.0).left(16.0).right(16.0).bottom(14.0));

    // Mini bar
    let bar: Option<Element<'_, Message>> = if !value.is_nan() {
        let max = if sensor.crit_val > 0.0 {
            sensor.crit_val * 1.2
        } else if sensor.warn_val > 0.0 {
            sensor.warn_val * 1.5
        } else {
            value.max(1.0) * 1.2
        };
        let pct = (value / max).clamp(0.0, 1.0);
        let bar_bg: Element<'_, Message> =
            container(Space::new(Length::Fill, Length::Fixed(4.0)))
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
        let bar_fill: Element<'_, Message> = container(Space::new(
            Length::FillPortion((pct * 100.0) as u16),
            Length::Fixed(4.0),
        ))
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
        Some(
            row![bar_fill, bar_bg]
                .spacing(0)
                .width(Length::Fill)
                .padding(Padding::new(0.0).left(16.0).right(16.0).bottom(16.0))
                .into(),
        )
    } else {
        None
    };

    let mut card_body = column![card_header, value_row];
    if let Some(b) = bar {
        card_body = card_body.push(b);
    }

    button(card_body)
        .width(Length::FillPortion(1))
        .style(theme::sensor_card_style(alarm))
        .on_press(Message::SelectSensor(sensor.id))
        .into()
}

// ---------------------------------------------------------------------------
// Status Bar (Footer)
// ---------------------------------------------------------------------------

fn status_bar<'a>() -> Element<'a, Message> {
    let left = row![
        text("Voltrus Lite").size(11).color(theme::CYAN),
        text("by").size(11).color(theme::TEXT_MUTED),
        text("JoyoDigitama").size(11).color(theme::CYAN),
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

    let right = row![live_dot, text("LIVE").size(11).color(theme::OK).font(theme::MONO)]
        .spacing(6)
        .align_y(Alignment::Center);

    container(
        row![left, Space::with_width(Length::Fill), right]
            .spacing(12)
            .align_y(Alignment::Center),
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
    })
    .into()
}

// ---------------------------------------------------------------------------
// Dialog helpers
// ---------------------------------------------------------------------------

fn delete_confirm(sensor_id: i64) -> Element<'static, Message> {
    let overlay = container(
        column![
            text("Delete Sensor").size(16).color(theme::TEXT),
            Space::with_height(8),
            text("Are you sure? This cannot be undone.")
                .size(13)
                .color(theme::TEXT_DIM),
            Space::with_height(16),
            row![
                button(text("Cancel").size(12))
                    .padding([6, 16])
                    .on_press(Message::CloseEditPanel)
                    .style(theme::header_btn_style()),
                button(text("Delete").size(12).color(theme::CRIT))
                    .padding([6, 16])
                    .on_press(Message::DeleteSensorConfirm(sensor_id))
                    .style(theme::danger_btn_style()),
            ]
            .spacing(8),
        ]
        .padding(24)
        .width(Length::Fixed(360.0)),
    )
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::CARD)),
        border: iced::border::Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .center(Length::Fill)
    .center_y(Length::Fill);

    overlay.into()
}

fn rename_screen_dialog<'a>(id: &'a str, name: &'a str) -> Element<'a, Message> {
    let input = text_input("Screen name", name)
        .on_input(move |v| Message::RenameScreenConfirm(id.to_string(), v))
        .padding(10)
        .style(theme::text_input_style);

    let overlay = container(
        column![
            text("Rename Screen").size(16).color(theme::TEXT),
            Space::with_height(12),
            input,
            Space::with_height(12),
            row![
                button(text("Cancel").size(12))
                    .padding([6, 16])
                    .on_press(Message::CloseEditPanel)
                    .style(theme::header_btn_style()),
                button(text("Rename").size(12))
                    .padding([6, 16])
                    .on_press(Message::RenameScreenConfirm(
                        id.to_string(),
                        name.to_string(),
                    ))
                    .style(theme::primary_btn_style()),
            ]
            .spacing(8),
        ]
        .padding(24)
        .width(Length::Fixed(360.0)),
    )
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::CARD)),
        border: iced::border::Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .center(Length::Fill)
    .center_y(Length::Fill);

    overlay.into()
}

fn delete_screen_dialog<'a>(id: &'a str, name: &'a str) -> Element<'a, Message> {
    let overlay = container(
        column![
            text(format!("Delete \"{}\"?", name))
                .size(16)
                .color(theme::TEXT),
            Space::with_height(8),
            text("This screen and its layout will be removed.")
                .size(13)
                .color(theme::TEXT_DIM),
            Space::with_height(16),
            row![
                button(text("Cancel").size(12))
                    .padding([6, 16])
                    .on_press(Message::CloseEditPanel)
                    .style(theme::header_btn_style()),
                button(text("Delete").size(12).color(theme::CRIT))
                    .padding([6, 16])
                    .on_press(Message::DeleteScreenConfirm(id.to_string()))
                    .style(theme::danger_btn_style()),
            ]
            .spacing(8),
        ]
        .padding(24)
        .width(Length::Fixed(360.0)),
    )
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::CARD)),
        border: iced::border::Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .center(Length::Fill)
    .center_y(Length::Fill);

    overlay.into()
}

fn create_screen_dialog(name: &str) -> Element<'_, Message> {
    let input_val = name.to_string();
    let overlay = container(
        column![
            text("New Screen").size(16).color(theme::TEXT),
            Space::with_height(12),
            text_input("Screen name", &input_val)
                .on_input(|v| {
                    // We can't easily update the dialog state from here,
                    // so just submit on input change
                    Message::CreateScreenConfirm(v)
                })
                .padding(10)
                .style(theme::text_input_style),
            Space::with_height(12),
            row![
                button(text("Cancel").size(12))
                    .padding([6, 16])
                    .on_press(Message::CloseEditPanel)
                    .style(theme::header_btn_style()),
            ]
            .spacing(8),
        ]
        .padding(24)
        .width(Length::Fixed(360.0)),
    )
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::CARD)),
        border: iced::border::Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .center(Length::Fill)
    .center_y(Length::Fill);

    overlay.into()
}
