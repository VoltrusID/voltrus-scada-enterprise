use iced::Element;
use crate::app::{App, Message};

pub fn view(_app: &App) -> Element<'_, Message> {
    iced::widget::text("Grid View - TODO").into()
}
