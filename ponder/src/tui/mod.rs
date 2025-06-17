use anyhow::{Context, Result};
use async_trait::async_trait;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::Ponder;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum AppState {
    MainScreen,
    DeckEdit,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum AppMode {
    Normal,
    Editing,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum EventResult {
    Normal,
    Quit,
}

#[async_trait]
pub(crate) trait Component {
    fn render(&mut self, frame: &mut Frame);
    async fn handle_event(&mut self, event: ()) -> Result<EventResult>;
}

#[derive(Debug)]
pub struct MainScreen<'a> {
    access: &'a Ponder,
    mode: AppMode,
    input_field: String,
}

impl<'a> MainScreen<'a> {
    pub fn new(ponder: &'a Ponder) -> Self {
        Self {
            access: ponder,
            mode: AppMode::Normal,
            input_field: String::new(),
        }
    }
}

#[async_trait]
impl<'a> Component for MainScreen<'a> {
    fn render(&mut self, frame: &mut Frame) {}

    async fn handle_event(&mut self, event: ()) -> Result<EventResult> {
        if let Event::Key(key) = event::read()? {
            let result = match self.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Esc => Ok(EventResult::Quit),
                    _ => Ok(EventResult::Normal),
                },
                AppMode::Editing => Ok(EventResult::Normal),
            };

            result
        } else {
            Ok(EventResult::Normal)
        }
    }
}

#[derive(Debug)]
pub struct Tui<'a> {
    state: AppState,
    terminal: DefaultTerminal,

    main_state: MainScreen<'a>,
}

impl<'a> Tui<'a> {
    pub fn new(store: &'a Ponder) -> Self {
        Self {
            state: AppState::MainScreen,
            terminal: ratatui::init(),
            main_state: MainScreen::new(&store),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<'a> Drop for Tui<'a> {
    fn drop(&mut self) {
        let _ = self
            .terminal
            .clear()
            .with_context(|| "clearing terminal on drop".to_string());

        ratatui::restore();
    }
}
