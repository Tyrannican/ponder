use anyhow::Result;
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
    pub(crate) id: Option<String>,
    pub(crate) object: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) color_indicator: Option<Vec<String>>,
    pub(crate) produced_mana: Option<Vec<String>>,
    pub(crate) loyalty: Option<String>,
    pub(crate) legalities: Option<HashMap<String, String>>,
    pub(crate) artist: Option<String>,
    pub(crate) oracle_id: Option<String>,
    pub(crate) type_line: Option<String>,
    pub(crate) defense: Option<String>,
    pub(crate) lang: Option<String>,
    pub(crate) card_faces: Option<Vec<ScryfallCard>>,
    pub(crate) content_warning: Option<bool>,
    pub(crate) cmc: Option<f32>,
    pub(crate) image_status: Option<String>,
    pub(crate) flavor_text: Option<String>,
    pub(crate) arena_id: Option<i32>,
    pub(crate) illustration_id: Option<String>,
    pub(crate) oracle_text: Option<String>,
    pub(crate) color_identity: Option<Vec<String>>,
    pub(crate) rarity: Option<String>,
    pub(crate) power: Option<String>,
    pub(crate) set_name: Option<String>,
    pub(crate) penny_rank: Option<i32>,
    pub(crate) variation: Option<bool>,
    pub(crate) set_id: Option<String>,
    pub(crate) toughness: Option<String>,
    pub(crate) mtgo_id: Option<i32>,
    pub(crate) colors: Option<Vec<String>>,
    pub(crate) booster: Option<bool>,
    pub(crate) border_color: Option<String>,
    pub(crate) foil: Option<bool>,
    pub(crate) set_type: Option<String>,
    pub(crate) nonfoil: Option<bool>,
    pub(crate) game_changer: Option<bool>,
    pub(crate) reprint: Option<bool>,
    pub(crate) layout: Option<String>,
    pub(crate) reserved: Option<bool>,
    pub(crate) digital: Option<bool>,
    pub(crate) set: Option<String>,
    pub(crate) keywords: Option<Vec<String>>,
    pub(crate) highres_image: Option<bool>,
    pub(crate) mana_cost: Option<String>,
    pub(crate) image_uris: Option<HashMap<String, String>>,
    pub(crate) games: Option<Vec<String>>,
    pub(crate) promo: Option<bool>,
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
    let valid = |card: &ScryfallCard| {
        if let Some(ref st) = card.set_type {
            if st == "vanguard" {
                return false;
            }
        }

        if let Some(ref sn) = card.set_name {
            if sn.contains("Mystery Booster Playtest") {
                return false;
            }
        }

        if let Some(ref g) = card.games {
            if g.contains(&"sega".to_string()) || g.contains(&"astral".to_string()) {
                return false;
            }
        }

        true
    };

    cards
        .into_iter()
        .filter(valid)
        .collect::<Vec<ScryfallCard>>()
}

pub async fn download_latest() -> Result<Vec<ScryfallCard>> {
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

#[cfg(test)]
mod scryfall_tests {
    use super::*;

    #[tokio::test]
    async fn checker() -> anyhow::Result<()> {
        let cards = download_latest().await?;
        Ok(())
    }
}
