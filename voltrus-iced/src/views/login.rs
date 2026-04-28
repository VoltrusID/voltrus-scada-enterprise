use iced::alignment;
use iced::widget::{button, column, container, text, text_input, Space};
use iced::{Element, Length};

use crate::app::{App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let title = text("Voltrus Viewer")
        .size(28)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center);

    let username_input = text_input("Username", &app.username)
        .on_input(Message::UsernameChanged)
        .padding(10);

    let password_input = text_input("Password", &app.password)
        .on_input(Message::PasswordChanged)
        .on_submit(Message::Login)
        .secure(true)
        .padding(10);

    let login_button = if app.login_loading {
        button(
            text("Logging in...")
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(10)
        .on_press_maybe(None)
    } else {
        button(
            text("Login")
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(10)
        .on_press(Message::Login)
    };

    let mut content = column![
        Space::with_height(80),
        title,
        Space::with_height(30),
        username_input,
        password_input,
        Space::with_height(10),
        login_button,
    ]
    .spacing(8)
    .max_width(350)
    .width(Length::Fill)
    .align_x(alignment::Horizontal::Center);

    if let Some(error) = &app.login_error {
        content = content.push(
            text(error)
                .color([0.9, 0.2, 0.2])
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
}
