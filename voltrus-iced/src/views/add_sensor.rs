use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Color, Element, Length};

use crate::app::{AddSensorField, AddSensorTab, App, Message};
use crate::theme;

pub fn view(app: &App) -> Element<'_, Message> {
    let form = &app.add_sensor_form;

    // -- Header --
    let title = text("Add Data Source").size(13).color(theme::TEXT);
    let close_btn = button(text("x").size(16).color(theme::TEXT_MUTED))
        .padding([2, 6])
        .on_press(Message::CloseAddSensor)
        .style(theme::icon_btn_style());

    let header = row![title, Space::with_width(Length::Fill), close_btn]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding([12, 16]);

    // -- Tabs --
    let existing_btn =
        button(text("Existing").size(12))
            .padding([8, 16])
            .style(theme::tab_button_style(matches!(
                app.add_sensor_tab,
                AddSensorTab::Existing
            )))
            .on_press(Message::AddSensorFormChanged(
                AddSensorField::SourceType,
                "__existing".into(),
            ));

    let new_btn = button(text("Create New").size(12))
        .padding([8, 16])
        .style(theme::tab_button_style(matches!(
            app.add_sensor_tab,
            AddSensorTab::New
        )))
        .on_press(Message::AddSensorFormChanged(
            AddSensorField::SourceType,
            "__new".into(),
        ));

    let tabs = row![existing_btn, new_btn].spacing(0);

    // -- Content --
    let content: Element<'_, Message> = match app.add_sensor_tab {
        AddSensorTab::Existing => existing_tab(app),
        AddSensorTab::New => new_sensor_tab(form),
    };

    let card_body = column![header, tabs, container(content).padding(16)]
        .width(Length::Fixed(500.0))
        .height(Length::Fixed(500.0));

    let card = container(card_body)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::CARD)),
            border: iced::border::Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        });

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.0, 0.0, 0.0, 0.6,
            ))),
            ..Default::default()
        })
        .into()
}

fn existing_tab(app: &App) -> Element<'_, Message> {
    let screen_ids: Vec<i64> = app
        .screen_sensors
        .iter()
        .map(|s| s.id)
        .collect();
    let available: Vec<&crate::models::SensorConfig> = app
        .all_sensors
        .iter()
        .filter(|s| !screen_ids.contains(&s.id))
        .collect();

    if available.is_empty() {
        return container(
            text("All sources already on this screen.")
                .size(13)
                .color(theme::TEXT_MUTED),
        )
        .padding(12)
        .into();
    }

    let mut list = column![].spacing(4);
    for s in &available {
        let card = button(
            row![
                row![
                    container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
                        .style(|_theme: &iced::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(Color::from_rgb(
                                0.545, 0.361, 0.965,
                            ))),
                            border: iced::border::Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }),
                    text(&s.name).size(13).color(theme::TEXT),
                    text(format!("({})", s.key))
                        .size(11)
                        .color(theme::TEXT_MUTED),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                Space::with_width(Length::Fill),
                text(&s.source_type).size(9).color(theme::TEXT_MUTED),
                text("+ Add").size(11).color(theme::CYAN),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .width(Length::Fill)
        .on_press(Message::AddExistingSensor(s.id))
        .style(theme::list_item_style());

        list = list.push(card);
    }

    list.into()
}

fn new_sensor_tab(form: &crate::app::AddSensorForm) -> Element<'_, Message> {
    let mut grid = column![].spacing(8);

    grid = grid.push(
        row![
            sensor_field("Key", &form.key, AddSensorField::Key),
            sensor_field("Name", &form.name, AddSensorField::Name),
        ]
        .spacing(8),
    );

    grid = grid.push(
        row![
            text("Data Source").size(10).color(theme::TEXT_MUTED),
            Space::with_width(Length::Fill),
        ]
        .spacing(0),
    );

    grid = grid.push(
        text_input("Source type", &form.source_type)
            .on_input(|v| Message::AddSensorFormChanged(AddSensorField::SourceType, v))
            .padding(8)
            .width(Length::Fill)
            .style(theme::text_input_style),
    );

    grid = grid.push(
        row![
            sensor_field("Unit", &form.unit, AddSensorField::Unit),
            sensor_num_field("Min", form.min_val, AddSensorField::MinVal),
        ]
        .spacing(8),
    );
    grid = grid.push(
        row![
            sensor_num_field("Max", form.max_val, AddSensorField::MaxVal),
            sensor_num_field("Base", form.base_val, AddSensorField::BaseVal),
        ]
        .spacing(8),
    );

    grid = grid.push(
        button(text("Create & Add").size(13).color(theme::CYAN))
            .padding([8, 16])
            .width(Length::Fill)
            .on_press(Message::CreateNewSensor)
            .style(theme::active_btn_style(theme::CYAN)),
    );

    grid.into()
}

fn sensor_field<'a>(
    label: &'a str,
    value: &'a str,
    field: AddSensorField,
) -> Element<'a, Message> {
    let label_el = text(label).size(10).color(theme::TEXT_MUTED);
    let input = text_input(label, value)
        .on_input(move |v| Message::AddSensorFormChanged(field, v))
        .padding(8)
        .style(theme::text_input_style);
    column![label_el, input].spacing(4).into()
}

fn sensor_num_field<'a>(
    label: &'a str,
    value: f64,
    field: AddSensorField,
) -> Element<'a, Message> {
    let label_el = text(label).size(10).color(theme::TEXT_MUTED);
    let input = text_input(label, &format!("{}", value))
        .on_input(move |v| Message::AddSensorFormChanged(field, v))
        .padding(8)
        .style(theme::text_input_style);
    column![label_el, input].spacing(4).into()
}
