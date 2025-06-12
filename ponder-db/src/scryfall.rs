use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, path::PathBuf};

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
    pub(crate) art_crop: Option<Cow<'a, str>>,
    pub(crate) png: Option<Cow<'a, str>>,
    pub(crate) normal: Option<Cow<'a, str>>,
    pub(crate) large: Option<Cow<'a, str>>,
    pub(crate) small: Option<Cow<'a, str>>,
    pub(crate) border_crop: Option<Cow<'a, str>>,
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

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = serde_json::to_string(&self).unwrap();
        write!(f, "{}", format.replace("\"", ""))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Legality {
    Legal,
    NotLegal,
    Banned,
    Restricted,
}

impl std::fmt::Display for Legality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = serde_json::to_string(&self).unwrap();
        write!(f, "{}", value.replace("\"", ""))
    }
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
    pub(crate) fn from_str(color: &Cow<'_, str>) -> Self {
        match color.trim() {
            "C" => Self::Colorless,
            "W" => Self::White,
            "U" => Self::Blue,
            "B" => Self::Black,
            "R" => Self::Red,
            "G" => Self::Green,
            "T" => Self::Tap,
            _ => panic!("unexpected char for color: {color}"),
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
    pub fn extract_types(&self) -> (Option<&str>, Option<Vec<&str>>, Option<Vec<&str>>) {
        let mut supertype: Option<&str> = None;
        let mut main_types: Option<Vec<&str>> = None;
        let mut subtypes: Option<Vec<&str>> = None;

        let Some(type_line) = self.type_line.as_ref() else {
            return (supertype, main_types, subtypes);
        };

        // Don't care about tokens
        if type_line.contains("Token") {
            return (supertype, main_types, subtypes);
        }

        let has_supertype = |phrase: &str| {
            matches!(
                phrase,
                "Legendary" | "Basic" | "Ongoing" | "Snow" | "World" | "Hero" | "Elite"
            )
        };

        let delim = "—";
        let split = type_line.split(" ").collect::<Vec<&str>>();
        let supertype_present = has_supertype(split[0]);
        match split.iter().position(|&s| s == delim) {
            Some(idx) => {
                if supertype_present {
                    supertype = Some(split[0]);
                    main_types = Some(split[1..idx].to_vec());
                } else {
                    main_types = Some(split[..idx].to_vec());
                }

                subtypes = Some(split[idx + 1..].to_vec());
            }
            None => {
                if supertype_present {
                    supertype = Some(split[0]);
                    main_types = Some(split[1..].to_vec());
                } else {
                    main_types = Some(split.to_vec());
                }
            }
        };

        (supertype, main_types, subtypes)
    }

    pub fn contains_game(&self, game: &str) -> bool {
        match self.games.as_ref() {
            Some(games) => games.contains(&Cow::Borrowed(game)),
            None => false,
        }
    }
}

async fn download_data<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    println!("Downloading latest card data...");
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

fn card_filter(card: &ScryfallCard) -> bool {
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

    if let Some(ref type_line) = card.type_line {
        if type_line.contains("Token") {
            return false;
        }
    }

    true
}

pub async fn download_latest<'a>() -> Result<Vec<ScryfallCard<'a>>> {
    // TODO: Temp
    let card_file = PathBuf::from("cards.json");
    let cards = if card_file.exists() {
        serde_json::from_str(&std::fs::read_to_string(&card_file)?)?
    } else {
        let bulk: BulkData = download_data::<BulkData>(URL).await?;
        let data = download_data::<Vec<ScryfallCard>>(&bulk.data[0].url).await?;
        let cards = data
            .into_iter()
            .filter(card_filter)
            .collect::<Vec<ScryfallCard>>();
        let out_str = serde_json::to_string_pretty(&cards)?;
        tokio::fs::write(&card_file, out_str).await?;

        cards
    };

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
