mod dependency;
mod app_state;

use crate::app_state::AppState;

fn main() -> iced::Result {
    iced::run("A cool counter", AppState::update, AppState::view)
}

