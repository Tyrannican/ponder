use color_eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

const URL: &str = "https://api.scryfall.com/bulk-data";

#[derive(Deserialize, Debug)]
struct BulkData {
    data: Vec<BulkEntry>,
}

#[derive(Deserialize, Debug)]
struct BulkEntry {
    #[serde(rename = "download_uri")]
    url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ScryfallCard {
    object: Option<String>,
    color_indicator: Option<Vec<String>>,
    produced_mana: Option<Vec<String>>,
    loyalty: Option<String>,
    // DB Table: Legalities - Card ID as FK
    legalities: Option<HashMap<String, String>>,
    artist: Option<String>,
    oracle_id: Option<String>,
    // This will need split into types (Legendary Creature - Elf Druid etc.)
    type_line: Option<String>,
    lang: Option<String>,
    // Double-face cards and such, treat the faces as separate cards
    card_faces: Option<Vec<ScryfallCard>>,
    // For racist stuff
    content_warning: Option<bool>,
    cmc: Option<f32>,
    image_status: Option<String>,
    flavor_text: Option<String>,
    arena_id: Option<i32>,
    illustration_id: Option<String>,
    oracle_text: Option<String>,
    color_identity: Option<Vec<String>>,
    rarity: Option<String>,
    power: Option<String>,
    set_name: Option<String>,
    penny_rank: Option<i32>,
    id: Option<String>,
    variation: Option<bool>,
    set_id: Option<String>,
    toughness: Option<String>,
    mtgo_id: Option<i32>,
    colors: Option<Vec<String>>,
    booster: Option<bool>,
    border_color: Option<String>,
    foil: Option<bool>,
    set_type: Option<String>,
    nonfoil: Option<bool>,
    // Commander only: OP cards
    game_changer: Option<bool>,
    reprint: Option<bool>,
    layout: Option<String>,
    reserved: Option<bool>,
    digital: Option<bool>,
    set: Option<String>,
    name: Option<String>,
    keywords: Option<Vec<String>>,
    highres_image: Option<bool>,
    mana_cost: Option<String>,
    // DB Table: Image URIs - Card ID as FK
    image_uris: Option<HashMap<String, String>>,
    games: Option<Vec<String>>,
    promo: Option<bool>,
}

async fn download_data<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let client = Client::new();
    let data = client
        .get(url)
        .header("accept", "application/json")
        .header("user-agent", "reqwest")
        .send()
        .await?
        .json::<T>()
        .await?;

    Ok(data)
}

// TODO: Better Filtering?
fn filter_cards(cards: Vec<ScryfallCard>) -> Vec<ScryfallCard> {
    let invalid = |card: &ScryfallCard| {
        match card.set_type {
            Some(ref st) => {
                if st == "vanguard" {
                    return true;
                }
            }
            None => {}
        }

        match card.set_name {
            Some(ref sn) => {
                if sn.contains("Mystery Booster Playtest") {
                    return true;
                }
            }
            None => {}
        }

        false
    };

    cards
        .into_iter()
        .filter_map(|c| {
            if !invalid(&c) {
                return Some(c);
            }

            None
        })
        .collect::<Vec<ScryfallCard>>()
}

pub async fn download() -> Result<Vec<ScryfallCard>> {
    // TODO: Temp
    let card_file = PathBuf::from("cards.json");
    let cards = if card_file.exists() {
        serde_json::from_str(&std::fs::read_to_string(&card_file)?)?
    } else {
        let bulk: BulkData = download_data::<BulkData>(URL).await?;
        let data = download_data::<Vec<ScryfallCard>>(&bulk.data[0].url).await?;
        let cards = filter_cards(data);
        let out_str = serde_json::to_string_pretty(&cards)?;
        std::fs::write(&card_file, out_str)?;

        cards
    };

    Ok(cards)
}
