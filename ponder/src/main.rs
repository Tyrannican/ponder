use std::path::PathBuf;

use anyhow::Result;
use dotstore;
use ponder_db::SqliteStore;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Layout, Position},
    style::Style,
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};

#[derive(Debug)]
pub struct Ponder {
    pub workspace: PathBuf,
    pub store: SqliteStore,

    input: String,
    char_idx: usize,
    results: Vec<String>,
}

impl Ponder {
    pub async fn new() -> Result<Self> {
        let workspace = dotstore::home_store("ponder")?.expect("could not find home directory");
        Ok(Self {
            store: SqliteStore::load(&workspace).await?,
            workspace,
            input: String::new(),
            char_idx: 0,
            results: Vec::new(),
        })
    }

    pub async fn run(&mut self, mut term: DefaultTerminal) -> Result<()> {
        loop {
            term.draw(|frame| self.draw(frame))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('c')
                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        return Ok(());
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.store.update().await?
                    }
                    KeyCode::Char(ch) => match self.enter_char(ch).await {
                        Ok(_) => {}
                        Err(e) => {
                            term.clear()?;
                            ratatui::restore();
                            println!("{e:?}");
                        }
                    },
                    KeyCode::Backspace => match self.delete_char().await {
                        Ok(_) => {}
                        Err(e) => {
                            term.clear()?;
                            ratatui::restore();
                            println!("{e:?}");
                        }
                    },
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    _ => {}
                }
            }
        }
    }

    async fn query(&mut self) -> Result<()> {
        if self.input.is_empty() {
            return Ok(());
        }

        if self.input.len() > 2 {
            let results = self.store.query_card_by_name(&self.input).await?;
            self.results = results.into_iter().map(|c| c.name).collect::<Vec<String>>();
        } else {
            self.results.clear();
        }

        Ok(())
    }

    async fn enter_char(&mut self, ch: char) -> Result<()> {
        let idx = self.byte_idx();
        self.input.insert(idx, ch);
        self.move_cursor_right();
        self.query().await
    }

    async fn delete_char(&mut self) -> Result<()> {
        let is_not_cursor_leftmost = self.char_idx != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.char_idx;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }

        self.query().await
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_idx.saturating_sub(1);
        self.char_idx = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_idx.saturating_add(1);
        self.char_idx = self.clamp_cursor(cursor_moved_right);
    }

    fn byte_idx(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_idx)
            .unwrap_or(self.input.len())
    }

    fn draw(&self, frame: &mut Frame) {
        let [entry, display] =
            Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(90)])
                .areas(frame.area());

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default())
            .block(Block::bordered().title("Enter card"));
        frame.render_widget(input, entry);
        frame.set_cursor_position(Position::new(
            entry.x + self.char_idx as u16 + 1,
            entry.y + 1,
        ));

        let results: Vec<ListItem> = self
            .results
            .iter()
            .map(|r| ListItem::new(Line::from(Span::raw(r))))
            .collect();

        let list = List::new(results).block(Block::bordered().title("Search Results"));
        frame.render_widget(list, display);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut state = Ponder::new().await?;
    let terminal = ratatui::init();
    state.run(terminal).await?;
    ratatui::restore();

    Ok(())
}
