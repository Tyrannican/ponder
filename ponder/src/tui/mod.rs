use anyhow::{Context, Result};
use async_trait::async_trait;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::Ponder;
use crate::data::Deck;

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
    Render,
    Quit,
}

#[async_trait]
pub(crate) trait Component {
    fn render(&mut self, frame: &mut Frame);
    async fn handle_event(&mut self, event: ()) -> Result<EventResult>;
}

#[derive(Debug)]
pub struct MainScreen<'a> {
    app: &'a Ponder,
    mode: AppMode,
    decks: Vec<Deck>,
}

impl<'a> MainScreen<'a> {
    pub fn new(ponder: &'a Ponder) -> Self {
        Self {
            app: ponder,
            mode: AppMode::Normal,
            decks: Vec::new(),
        }
    }

    pub fn load_all_decks(&self) -> Result<()> {
        //
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
                    _ => Ok(EventResult::Render),
                },
                AppMode::Editing => Ok(EventResult::Render),
            };

            result
        } else {
            Ok(EventResult::Render)
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
