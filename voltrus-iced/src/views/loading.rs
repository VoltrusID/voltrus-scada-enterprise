use iced::widget::{container, column, text, Space};
use iced::{Element, Length};

use crate::theme;

pub fn view() -> Element<'static, crate::app::Message> {
    container(
        column![
            Space::with_height(100),
            text("Loading...")
                .size(18)
                .color(theme::TEXT_MUTED),
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
