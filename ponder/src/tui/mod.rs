use anyhow::{Context, Result};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
};

use crate::Ponder;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum AppState {
    MainScreen,
    DeckEdit,
}

#[derive(Debug)]
pub struct Tui {
    ponder: Ponder,
    state: AppState,
    terminal: DefaultTerminal,
}

impl Tui {
    pub fn new(store: Ponder) -> Self {
        Self {
            ponder: store,
            state: AppState::MainScreen,
            terminal: ratatui::init(),
        }
    }

    pub fn draw(&self) {
        // Get a Layout depending on State
        // Render it
    }

    pub async fn parse_input(&mut self) -> Result<bool> {
        Ok(false)
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            self.draw();
            match self.parse_input().await {
                Ok(quit) => {
                    if quit {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self
            .terminal
            .clear()
            .with_context(|| "clearing terminal on drop".to_string());

        ratatui::restore();
    }
}
