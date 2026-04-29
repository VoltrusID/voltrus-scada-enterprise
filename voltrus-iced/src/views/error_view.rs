use iced::widget::{button, column, container, text, Space};
use iced::{Element, Length};

use crate::app::Message;
use crate::theme;

pub fn view(msg: &str) -> Element<'_, Message> {
    container(
        column![
            Space::with_height(100),
            text(format!("Error: {msg}"))
                .size(18)
                .color(theme::CRIT),
            Space::with_height(20),
            button(text("Back to Login").size(13))
                .padding([8, 16])
                .style(theme::header_btn_style())
                .on_press(Message::Logout),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(theme::BG)),
        ..Default::default()
    })
    .into()
}
