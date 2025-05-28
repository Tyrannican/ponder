use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, path::PathBuf};
use tokio::io::split;

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
pub(crate) struct ImageUris<'a> {
    art_crop: Option<Cow<'a, str>>,
    png: Option<Cow<'a, str>>,
    normal: Option<Cow<'a, str>>,
    large: Option<Cow<'a, str>>,
    small: Option<Cow<'a, str>>,
    border_crop: Option<Cow<'a, str>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Format {
    Standard,
    Future,
    Historic,
    Timeless,
    Gladiator,
    Pioneer,
    Explorer,
    Modern,
    Legacy,
    Pauper,
    Vintage,
    Penny,
    Commander,
    Oathbreaker,
    StandardBrawl,
    Brawl,
    Alchemy,
    PauperCommander,
    Duel,
    Oldschool,
    Premodern,
    Predh,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Legality {
    Legal,
    NotLegal,
    Banned,
    Restricted,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Color {
    Colorless = 0,
    White = 1,
    Blue = 2,
    Black = 4,
    Red = 8,
    Green = 16,
    Tap = 32, // Only ONE card has this and it's an Unfinity card
}

impl Color {
    pub(crate) fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub(crate) fn from_char(ch: char) -> Self {
        match ch {
            'C' => Self::Colorless,
            'W' => Self::White,
            'U' => Self::Blue,
            'B' => Self::Black,
            'R' => Self::Red,
            'G' => Self::Green,
            'T' => Self::Tap,
            _ => panic!("unexpected char for color: {ch}"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ScryfallCard<'a> {
    pub(crate) id: Option<Cow<'a, str>>,
    pub(crate) object: Cow<'a, str>,
    pub(crate) name: Cow<'a, str>,
    pub(crate) color_indicator: Option<Vec<Cow<'a, str>>>,
    pub(crate) loyalty: Option<Cow<'a, str>>,
    pub(crate) legalities: Option<BTreeMap<Format, Legality>>,
    pub(crate) artist: Option<Cow<'a, str>>,
    pub(crate) oracle_id: Option<Cow<'a, str>>,
    pub(crate) type_line: Option<Cow<'a, str>>,
    pub(crate) defense: Option<Cow<'a, str>>,
    pub(crate) lang: Option<Cow<'a, str>>,
    pub(crate) card_faces: Option<Vec<ScryfallCard<'a>>>,
    pub(crate) content_warning: Option<bool>,
    pub(crate) cmc: Option<f32>,
    pub(crate) image_status: Option<Cow<'a, str>>,
    pub(crate) flavor_text: Option<Cow<'a, str>>,
    pub(crate) arena_id: Option<i32>,
    pub(crate) illustration_id: Option<Cow<'a, str>>,
    pub(crate) oracle_text: Option<Cow<'a, str>>,
    pub(crate) colors: Option<Vec<Cow<'a, str>>>,
    pub(crate) color_identity: Option<Vec<Cow<'a, str>>>,
    pub(crate) produced_mana: Option<Vec<Cow<'a, str>>>,
    pub(crate) rarity: Option<Cow<'a, str>>,
    pub(crate) power: Option<Cow<'a, str>>,
    pub(crate) set_name: Option<Cow<'a, str>>,
    pub(crate) penny_rank: Option<i32>,
    pub(crate) variation: Option<bool>,
    pub(crate) set_id: Option<Cow<'a, str>>,
    pub(crate) toughness: Option<Cow<'a, str>>,
    pub(crate) mtgo_id: Option<i32>,
    pub(crate) booster: Option<bool>,
    pub(crate) border_color: Option<Cow<'a, str>>,
    pub(crate) foil: Option<bool>,
    pub(crate) set_type: Option<Cow<'a, str>>,
    pub(crate) nonfoil: Option<bool>,
    pub(crate) game_changer: Option<bool>,
    pub(crate) reprint: Option<bool>,
    pub(crate) layout: Option<Cow<'a, str>>,
    pub(crate) reserved: Option<bool>,
    pub(crate) digital: Option<bool>,
    pub(crate) set: Option<Cow<'a, str>>,
    pub(crate) keywords: Option<Vec<Cow<'a, str>>>,
    pub(crate) highres_image: Option<bool>,
    pub(crate) mana_cost: Option<Cow<'a, str>>,
    pub(crate) image_uris: Option<ImageUris<'a>>,
    pub(crate) games: Option<Vec<Cow<'a, str>>>,
    pub(crate) promo: Option<bool>,
}

impl<'a> ScryfallCard<'a> {
    // We need to deal with the following per card:
    // * Supertype: (Basic Legendary, Ongoing, Snow, World) - https://mtg.fandom.com/wiki/Supertype
    // * Type: The main type of the card (Artifact, Enchantment, Sorcery, etc)
    // * Subtype: Anything after the - in the `type_line` field (Delimited by space)
    //
    // We also need to deal with Double-faced cards (i.e. any card with multiple card faces)
    pub fn extract_types(&self) {
        if self.card_faces.is_some() {
            for face in self.card_faces.as_ref().unwrap() {
                face.extract_types();
            }

            return;
        }

        let Some(type_line) = self.type_line.as_ref() else {
            return;
        };

        // Don't care about tokens
        if type_line.contains("Token") {
            return;
        }

        let has_supertype = |phrase: &str| {
            matches!(
                phrase,
                "Legendary" | "Basic" | "Ongoing" | "Snow" | "World" | "Hero" | "Elite"
            )
        };
        let delim = " — ";
        let (main_types, subtypes) = type_line.split_once(delim).unwrap_or((type_line, ""));
        let (super_type, main_type) = main_types.split_once(" ").unwrap_or(("", main_types));
        println!("Supertype: {super_type} Main Type: {main_type} Subtype: {subtypes}");
    }
}

async fn download_data<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let data = Client::new()
        .get(url)
        .header("accept", "application/json")
        .header("user-agent", "reqwest")
        .send()
        .await?
        .json::<T>()
        .await?;

    Ok(data)
}

// TODO: Extract out when it becomes a bit too much
fn filter_cards(cards: Vec<ScryfallCard>) -> Vec<ScryfallCard> {
    let valid = |card: &ScryfallCard| {
        // Weird vanguard cards
        if let Some(ref st) = card.set_type {
            if st == "vanguard" {
                return false;
            }
        }

        // Test play cards - there are probably others
        if let Some(ref sn) = card.set_name {
            if sn.contains("Mystery Booster Playtest") {
                return false;
            }
        }

        // Art cards
        if let Some(ref type_line) = card.type_line {
            if type_line.contains("Card") {
                return false;
            }
        }

        // Unsupported formats (i.e. 90s promotions)
        if let Some(ref g) = card.games {
            if g.contains(&Cow::Borrowed("sega")) || g.contains(&Cow::Borrowed("astral")) {
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

pub async fn download_latest<'a>() -> Result<Vec<ScryfallCard<'a>>> {
    // TODO: Temp
    let card_file = PathBuf::from("cards.json");
    let cards = if card_file.exists() {
        serde_json::from_str(&std::fs::read_to_string(&card_file)?)?
    } else {
        let bulk: BulkData = download_data::<BulkData>(URL).await?;
        let data = download_data::<Vec<ScryfallCard>>(&bulk.data[0].url).await?;
        let cards = filter_cards(data);
        let out_str = serde_json::to_string_pretty(&cards)?;
        tokio::fs::write(&card_file, out_str).await?;

        cards
    };

    for card in cards.iter() {
        card.extract_types();
    }

    Ok(cards)
}

#[cfg(test)]
mod scryfall_tests {
    use super::*;

    #[tokio::test]
    async fn checker() -> anyhow::Result<()> {
        download_latest().await?;
        Ok(())
    }
}
