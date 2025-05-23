use crate::ScryfallCard;
use sqlx::FromRow;

#[derive(Debug, FromRow, Default)]
pub struct Card {
    id: String,
    object: String,
    name: String,
    color_indicator: Option<String>,
    produced_mana: Option<String>,
    loyalty: Option<i32>,
    artist: Option<String>,
    oracle_id: Option<String>,
    lang: Option<String>,
    content_warning: Option<bool>,
    converted_mana_cost: Option<f32>,
    image_status: Option<String>,
    flavor_tex: Option<String>,
    arena_id: Option<i32>,
    illustration_id: Option<String>,
    oracle_text: Option<String>,
    color_identity: Option<String>,
    rarity: Option<String>,
    card_type: Option<String>,
    subtype: Option<String>,
    legendary: Option<bool>,
    power: Option<i32>,
    toughness: Option<i32>,
    set_name: Option<String>,
    set_id: Option<String>,
    set_type: Option<String>,
    set_short: Option<String>,
    penny_rank: Option<String>,
    variation: Option<bool>,
    mtgo_id: Option<i32>,
    colors: Option<String>,
    booster: Option<bool>,
    border_color: Option<String>,
    foil: Option<bool>,
    game_changer: Option<bool>,
    reprint: Option<bool>,
    layout: Option<String>,
    reserved: Option<bool>,
    digital: Option<bool>,
    keywords: Option<String>,
    mana_cost: Option<String>,
    mtgo: Option<bool>,
    arena: Option<bool>,
    paper: Option<bool>,
    promo: Option<bool>,
}

impl Card {
    pub fn from_scryfall(card: ScryfallCard) -> Vec<Card> {
        if let Some(card_faces) = card.card_faces {
            return card_faces
                .into_iter()
                .map(|c| Self::from_scryfall(c))
                .flatten()
                .collect::<Vec<Card>>();
        }

        let mut new_card = Self::default();
        if let Some(type_line) = card.type_line {
            // Some cards are just art etc.
            if type_line == "Card" {
                return Vec::new();
            }

            let Some((split, rest)) = type_line.split_once(" — ") else {
                // println!("{type_line}");
                return Vec::new();
            };

            let inner = split.split(" ").collect::<Vec<&str>>();
            if inner.len() > 1 {
                println!("{inner:?}");
            }
        }

        vec![Self::default()]
    }
}
