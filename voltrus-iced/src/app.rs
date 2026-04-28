use iced::{Element, Subscription, Task};

pub struct App;

#[derive(Debug, Clone)]
pub enum Message {}

pub fn new() -> (App, Task<Message>) {
    (App, Task::none())
}

pub fn update(_state: &mut App, _message: Message) -> Task<Message> {
    Task::none()
}

pub fn view(_state: &App) -> Element<'_, Message> {
    iced::widget::text("Voltrus Viewer").into()
}

pub fn subscription(_state: &App) -> Subscription<Message> {
    Subscription::none()
}
