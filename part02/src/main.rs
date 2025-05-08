mod dependency;
mod app_state;

use crate::app_state::AppState;

fn main() {
    let app = iced::application("dependecy_analyzer", AppState::update, AppState::view)
        .centered()
        .theme(|_| iced::Theme::Dark)
        .subscription(AppState::subscription)
        .antialiasing(false);
    let _ = app.run();
}

