use std::path::PathBuf;

use anyhow::{Context, Result};
use dotstore;
use ponder_db::SqliteStore;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
};

mod tui;

#[derive(Debug)]
pub struct Ponder {
    pub workspace: PathBuf,
    pub store: SqliteStore,
    pub terminal: DefaultTerminal,
}

impl Ponder {
    pub async fn new() -> Result<Self> {
        let workspace = dotstore::home_store("ponder")
            .with_context(|| "finding home directory".to_string())?
            .with_context(|| "home directory not set")?;

        Ok(Self {
            store: SqliteStore::load(&workspace).await?,
            workspace,
            terminal: ratatui::init(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            // self.terminal.draw(|frame| self.render(frame))?;
            match self.parse_input().await {
                Ok(quit) => {
                    if quit {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("{e:#?}");
                }
            }
        }

        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        //
    }

    async fn parse_input(&mut self) -> Result<bool> {
        Ok(false)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut state = Ponder::new().await?;
    let mut terminal = ratatui::init();
    match state.run().await {
        Ok(_) => {}
        Err(e) => {
            terminal.clear()?;
            ratatui::restore();
            println!("{e:?}");

            return Err(e);
        }
    }

    ratatui::restore();

    Ok(())
}
