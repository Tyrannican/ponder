use ponder_db::{
    card::Card,
    scryfall::{Color, Format},
};

pub type DeckEntry = (Card, u8);

#[derive(Debug)]
pub struct Deck {
    pub format: Format,
    pub name: String,
    pub commander: Option<String>,
    pub colors: Vec<Color>,
    pub cards: Vec<DeckEntry>,
}
