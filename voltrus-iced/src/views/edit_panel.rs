use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::app::{App, EditField, EditTab, Message};
use crate::models;
use crate::theme;

const CHART_TYPES: &[&str] = &["sparkline", "bignumber", "gauge", "image", "text"];

pub fn view(app: &App) -> Element<'_, Message> {
    let form = match &app.edit_form {
        Some(f) => f,
        None => return Space::new(Length::Fill, Length::Fill).into(),
    };

    // -- Header --
    let title = text(format!("Edit: {}", form.name))
        .size(13)
        .color(theme::TEXT);

    let source_tag = text(&form.source_type)
        .size(10)
        .color(theme::TEXT_MUTED)
        .font(theme::MONO);

    let close_btn = button(text("x").size(16).color(theme::TEXT_MUTED))
        .padding([2, 6])
        .on_press(Message::CloseEditPanel)
        .style(theme::icon_btn_style());

    let header = row![
        title,
        Space::with_width(Length::Fill),
        source_tag,
        close_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(Padding::new(12.0).left(16.0).right(16.0));

    // -- Tabs --
    let general_btn = button(text("General").size(12))
        .padding([8, 16])
        .style(theme::tab_button_style(matches!(app.edit_tab, EditTab::General)))
        .on_press(Message::EditTabChanged(EditTab::General));

    let source_btn = button(text("Source").size(12))
        .padding([8, 16])
        .style(theme::tab_button_style(matches!(app.edit_tab, EditTab::Source)))
        .on_press(Message::EditTabChanged(EditTab::Source));

    let tabs = row![general_btn, source_btn].spacing(0);

    // -- Form content --
    let content: Element<'_, Message> = match app.edit_tab {
        EditTab::General => general_tab(form),
        EditTab::Source => source_tab(form),
    };

    // -- Footer --
    let save_btn = button(text("Save").size(13).color(theme::CYAN))
        .padding([6, 16])
        .on_press(Message::SaveSensor)
        .style(theme::active_btn_style(theme::CYAN));

    let delete_btn = button(text("Delete").size(13).color(theme::CRIT))
        .padding([6, 16])
        .on_press(Message::DeleteSensorConfirm(form.id))
        .style(theme::danger_btn_style());

    let footer = row![save_btn, delete_btn]
        .spacing(8)
        .padding(Padding::new(16.0).top(0.0));

    // -- Assemble card --
    let card_body = column![
        header,
        tabs,
        container(content)
            .padding(16)
            .height(Length::Fill),
        footer,
    ]
    .width(Length::Fixed(480.0))
    .height(Length::Fixed(520.0));

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

fn general_tab(form: &models::SensorConfig) -> Element<'_, Message> {
    let mut grid = column![].spacing(8);

    // Name + Key row
    grid = grid.push(
        row![
            field("Name", &form.name, EditField::Name),
            field("Key", &form.key, EditField::Key),
        ]
        .spacing(8),
    );

    // Unit
    grid = grid.push(field("Unit", &form.unit, EditField::Unit));

    // Numeric fields
    grid = grid.push(
        row![
            num_field("Min", form.min_val, EditField::MinVal),
            num_field("Max", form.max_val, EditField::MaxVal),
        ]
        .spacing(8),
    );
    grid = grid.push(
        row![
            num_field("Warn", form.warn_val, EditField::WarnVal),
            num_field("Crit", form.crit_val, EditField::CritVal),
        ]
        .spacing(8),
    );
    grid = grid.push(
        row![
            num_field("Base", form.base_val, EditField::BaseVal),
            num_field("Decimals", form.decimals as f64, EditField::Decimals),
        ]
        .spacing(8),
    );

    // Color + Chart type (use text input for chart_type)
    grid = grid.push(
        row![
            field("Color", &form.color, EditField::Color),
            field("Chart Type", &form.chart_type, EditField::ChartType),
        ]
        .spacing(8),
    );

    grid.into()
}

fn source_tab(form: &models::SensorConfig) -> Element<'_, Message> {
    let mut col = column![].spacing(8);

    col = col.push(text("Data Source").size(11).color(theme::TEXT_MUTED));

    // Use text input for source type (pick_list has different style signature)
    col = col.push(
        text_input("Source type", &form.source_type)
            .on_input(|v| Message::EditFieldChanged(EditField::SourceType, v))
            .padding(8)
            .style(theme::text_input_style),
    );

    col = col.push(text("Source config (JSON)").size(11).color(theme::TEXT_MUTED));

    col = col.push(
        text_input("JSON config", &form.source_config)
            .on_input(|v| Message::EditFieldChanged(EditField::SourceConfig, v))
            .padding(8)
            .style(theme::text_input_style),
    );

    col.into()
}

fn field<'a>(label: &'a str, value: &'a str, field: EditField) -> Element<'a, Message> {
    let label_el = text(label)
        .size(10)
        .color(theme::TEXT_MUTED)
        .width(Length::Fill);
    let input = text_input(label, value)
        .on_input(move |v| Message::EditFieldChanged(field, v))
        .padding(8)
        .style(theme::text_input_style);
    column![label_el, input].spacing(4).into()
}

fn num_field<'a>(label: &'a str, value: f64, field: EditField) -> Element<'a, Message> {
    let label_el = text(label)
        .size(10)
        .color(theme::TEXT_MUTED)
        .width(Length::Fill);
    let input = text_input(label, &format!("{:.2}", value))
        .on_input(move |v| Message::EditFieldChanged(field, v))
        .padding(8)
        .style(theme::text_input_style);
    column![label_el, input].spacing(4).into()
}
