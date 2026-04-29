use iced::alignment;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::app::{App, Message};
use crate::theme;

pub fn view(app: &App) -> Element<'_, Message> {
    let logo = container(
        text("V")
            .size(18)
            .color(theme::BG)
            .font(theme::MONO),
    )
    .width(36)
    .height(36)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::OK)),
        border: iced::border::Border {
            color: theme::OK,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    let title = text(&app.branding.title)
        .size(18)
        .color(theme::TEXT)
        .font(iced::Font::DEFAULT);

    let header = row![logo, title]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    let username_input = text_input("Enter username", &app.username)
        .on_input(Message::UsernameChanged)
        .padding(Padding::new(10.0))
        .style(theme::text_input_style);

    let username_label = text("Username")
        .size(11)
        .color(theme::TEXT_MUTED);

    let password_input = text_input("Enter password", &app.password)
        .on_input(Message::PasswordChanged)
        .on_submit(Message::Login)
        .secure(true)
        .padding(Padding::new(10.0))
        .style(theme::text_input_style);

    let password_label = text("Password")
        .size(11)
        .color(theme::TEXT_MUTED);

    let login_button = if app.login_loading {
        button(
            text("Signing in...")
                .size(14)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(10)
        .style(theme::primary_btn_style())
    } else {
        button(
            text("Sign In")
                .size(14)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(10)
        .on_press(Message::Login)
        .style(theme::primary_btn_style())
    };

    let mut content = column![
        header,
        Space::with_height(24),
        username_label,
        Space::with_height(4),
        username_input,
        Space::with_height(12),
        password_label,
        Space::with_height(4),
        password_input,
        Space::with_height(16),
        login_button,
    ]
    .spacing(0)
    .max_width(340)
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Left);

    if let Some(error) = &app.login_error {
        content = content.push(
            column![
                Space::with_height(12),
                text(error)
                    .size(12)
                    .color(theme::CRIT)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Left),
            ],
        );
    }

    let card = container(content)
        .width(360)
        .padding(32)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::CARD)),
            border: iced::border::Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        });

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::BG)),
            ..Default::default()
        })
        .into()
}
