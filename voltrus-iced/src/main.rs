mod app;

pub fn main() -> iced::Result {
    iced::application("Voltrus Viewer", app::update, app::view)
        .subscription(app::subscription)
        .theme(|_state| iced::Theme::Dark)
        .window_size((1024.0, 700.0))
        .run_with(app::new)
}
