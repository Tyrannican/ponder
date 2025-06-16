use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
};

mod state;
use crate::Ponder;
use state::AppState;

#[derive(Debug)]
pub struct Tui {
    store: Ponder,
    state: AppState,
    terminal: DefaultTerminal,
}

impl Tui {
    pub fn new(store: Ponder) -> Self {
        Self {
            store,
            state: AppState::MainScreen,
            terminal: ratatui::init(),
        }
    }
}
