use crate::scryfall::{Format, ScryfallCard, download_latest};
use anyhow::{Context, Result};
use sqlx::{Row, SqliteTransaction, sqlite::SqlitePool};

macro_rules! insert_image {
    ($card:expr, $txn:expr, $images:expr, $field:ident, $card_id:expr) => {
        if let Some(ref uri) = $images.$field {
            let img_id: i64 = sqlx::query_scalar("select id from image_type where name = ?")
                .bind(stringify!($field))
                .fetch_one($txn.as_mut())
                .await
                .with_context(|| format!("fetching id for {} image_type", stringify!($field)))?;

            sqlx::query("insert or ignore into images(card_id,image_type_id,uri) values(?, ?, ?)")
                .bind($card_id)
                .bind(img_id)
                .bind(&uri)
                .execute($txn.as_mut())
                .await
                .with_context(|| {
                    format!(
                        "inserting image uri - {uri} {}",
                        $card.name.as_ref().unwrap()
                    )
                })?;
        }
    };
}

macro_rules! colors_as_u8 {
    ($card:expr, $field:ident) => {
        if let Some(ref values) = $card.$field {
            let mut value: u8 = 0;
            for color in values.iter() {
                let color = crate::scryfall::Color::from_str(color);
                value += color as u8;
            }

            Some(value)
        } else {
            None
        }
    };
}

macro_rules! string_to_integer {
    ($card:expr, $field:ident) => {
        if let Some(ref value) = $card.$field {
            match value.parse::<i32>() {
                Ok(v) => Some(v),
                Err(_) => None,
            }
        } else {
            None
        }
    };
}

#[derive(Debug, Clone)]
pub struct DatabaseUpdater<'a> {
    pool: &'a SqlitePool,
}

// TODO: Deal with Card faces being ignored when added
impl<'a> DatabaseUpdater<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn update(&self) -> Result<()> {
        println!("Updating Database...");
        let cards = download_latest().await?;

        let mut txn = self.pool.begin().await?;
        self.add_formats(&mut txn).await?;
        self.add_image_types(&mut txn).await?;
        txn.commit().await?;

        for batch in cards.chunks(1000) {
            let mut txn = self.pool.begin().await?;
            for card in batch.iter() {
                if let Some(ref faces) = card.card_faces {
                    for face in faces.iter() {
                        self.add_card(&face, &mut txn).await?;
                        self.add_legalities(&face, &mut txn).await?;
                        self.add_keywords(&face, &mut txn).await?;
                        self.add_images(&face, &mut txn).await?;
                        self.add_card_types(&face, &mut txn).await?;
                    }
                } else {
                    self.add_card(&card, &mut txn).await?;
                    self.add_legalities(&card, &mut txn).await?;
                    self.add_keywords(&card, &mut txn).await?;
                    self.add_images(&card, &mut txn).await?;
                    self.add_card_types(&card, &mut txn).await?;
                }
            }
            txn.commit().await?;
        }

        Ok(())
    }

    async fn add_formats(&self, txn: &mut SqliteTransaction<'_>) -> Result<()> {
        for format in [
            Format::Modern,
            Format::StandardBrawl,
            Format::Oathbreaker,
            Format::Commander,
            Format::Penny,
            Format::Brawl,
            Format::PauperCommander,
            Format::Alchemy,
            Format::Premodern,
            Format::Predh,
            Format::Oldschool,
            Format::Vintage,
            Format::Legacy,
            Format::Historic,
            Format::Future,
            Format::Standard,
            Format::Pauper,
            Format::Timeless,
            Format::Pioneer,
            Format::Gladiator,
            Format::Explorer,
            Format::Duel,
        ] {
            sqlx::query("insert or ignore into format(name) values(?)")
                .bind(&format.to_string())
                .execute(txn.as_mut())
                .await
                .with_context(|| format!("inserting {format:?} into db"))?;
        }
        Ok(())
    }

    async fn add_image_types(&self, txn: &mut SqliteTransaction<'_>) -> Result<()> {
        for image_type in ["art_crop", "png", "normal", "large", "small", "border_crop"] {
            sqlx::query("insert or ignore into image_type(name) values(?)")
                .bind(image_type)
                .execute(txn.as_mut())
                .await
                .with_context(|| format!("inserting image type - {image_type}"))?;
        }

        Ok(())
    }

    // Insert card into all tables with insert or ignore
    async fn add_card(
        &self,
        card: &ScryfallCard<'_>,
        txn: &mut SqliteTransaction<'_>,
    ) -> Result<()> {
        let query = r#"
            insert or ignore into card(
                card_id,
                object,
                name,
                color_indicator,
                produced_mana,
                loyalty,
                artist,
                oracle_id,
                type_line,
                lang,
                content_warning,
                converted_mana_cost,
                image_status,
                flavor_text,
                arena_id,
                illustration_id,
                oracle_text,
                colors,
                color_identity,
                rarity,
                power,
                toughness,
                set_name,
                set_id, 
                set_type, 
                set_short, 
                penny_rank,
                variation,
                mtgo_id,
                booster,
                border_color,
                foil,
                game_changer,
                reprint,
                layout,
                reserved,
                digital,
                mana_cost,
                mtgo,
                arena,
                paper,
                promo 
            ) values(
                ?1,
                ?2,
                ?3,
                ?4,
                ?5,
                ?6,
                ?7,
                ?8,
                ?9,
                ?10,
                ?11,
                ?12,
                ?13,
                ?14,
                ?15,
                ?16,
                ?17,
                ?18,
                ?19,
                ?20,
                ?21,
                ?22,
                ?23,
                ?24,
                ?25,
                ?26,
                ?27,
                ?28,
                ?29,
                ?30,
                ?31,
                ?32,
                ?33,
                ?34,
                ?35,
                ?36,
                ?37,
                ?38,
                ?39,
                ?40,
                ?41,
                ?42
            )
        "#;

        sqlx::query(&query)
            .bind(&card.id)
            .bind(&card.object)
            .bind(&card.name)
            .bind(colors_as_u8!(card, color_indicator))
            .bind(colors_as_u8!(card, produced_mana))
            .bind(&card.loyalty)
            .bind(&card.artist)
            .bind(&card.oracle_id)
            .bind(&card.type_line)
            .bind(&card.lang)
            .bind(&card.content_warning)
            .bind(&card.cmc)
            .bind(&card.image_status)
            .bind(&card.flavor_text)
            .bind(&card.arena_id)
            .bind(&card.illustration_id)
            .bind(&card.oracle_text)
            .bind(colors_as_u8!(card, colors))
            .bind(colors_as_u8!(card, color_identity))
            .bind(&card.rarity)
            .bind(string_to_integer!(card, power))
            .bind(string_to_integer!(card, toughness))
            .bind(&card.set_name)
            .bind(&card.set_id)
            .bind(&card.set_type)
            .bind(&card.set)
            .bind(&card.penny_rank)
            .bind(&card.variation)
            .bind(&card.mtgo_id)
            .bind(&card.booster)
            .bind(&card.border_color)
            .bind(&card.foil)
            .bind(&card.game_changer)
            .bind(&card.reprint)
            .bind(&card.layout)
            .bind(&card.reserved)
            .bind(&card.digital)
            .bind(&card.mana_cost)
            .bind(&card.contains_game("mtgo"))
            .bind(&card.contains_game("arena"))
            .bind(&card.contains_game("paper"))
            .bind(&card.promo)
            .execute(txn.as_mut())
            .await
            .with_context(|| {
                format!(
                    "insert {} into database - {}",
                    card.name.as_ref().unwrap(),
                    card.id.as_ref().unwrap()
                )
            })?;

        Ok(())
    }

    async fn add_legalities(
        &self,
        card: &ScryfallCard<'_>,
        txn: &mut SqliteTransaction<'_>,
    ) -> Result<()> {
        if let Some(legalities) = &card.legalities {
            for (format, legality) in legalities.iter() {
                let (format_id, card_id): (i64, i64) = sqlx::query_as(
                    "select f.id, c.id from format f, card c where f.name = ? and c.card_id = ?",
                )
                .bind(&format.to_string())
                .bind(&card.id)
                .fetch_one(txn.as_mut())
                .await?;

                sqlx::query(
                    "insert or ignore into legality(card_id, format_id, status) values(?, ?, ?)",
                )
                .bind(card_id)
                .bind(format_id)
                .bind(&legality.to_string())
                .execute(txn.as_mut())
                .await
                .with_context(|| {
                    format!(
                        "inserting legality {} ({}) {} ({}) {}\n{card:?}",
                        card.id.as_ref().unwrap(),
                        card.name.as_ref().unwrap(),
                        format_id,
                        format,
                        legality
                    )
                })?;
            }
        }

        Ok(())
    }

    async fn add_keywords(
        &self,
        card: &ScryfallCard<'_>,
        txn: &mut SqliteTransaction<'_>,
    ) -> Result<()> {
        let card_id: i64 = sqlx::query_scalar("select id from card where card_id = ?")
            .bind(&card.id)
            .fetch_one(txn.as_mut())
            .await
            .with_context(|| {
                format!(
                    "fetching card id (keyword) - {}",
                    card.name.as_ref().unwrap()
                )
            })?;

        if let Some(ref keywords) = card.keywords {
            for keyword in keywords.iter() {
                let row = sqlx::query(
                    "insert into keyword(name) values(?) on conflict do nothing returning id",
                )
                .bind(keyword)
                .fetch_optional(txn.as_mut())
                .await
                .with_context(|| format!("inserting keyword - {keyword}"))?;

                let keyword_id: i64 = match row {
                    Some(r) => r.get("id"),
                    None => sqlx::query_scalar("select id from keyword where name = ?")
                        .bind(keyword)
                        .fetch_one(txn.as_mut())
                        .await
                        .with_context(|| format!("fetching keyword id: {keyword}"))?,
                };

                sqlx::query(
                    "insert or ignore into card_keywords(card_id, keyword_id) values(?, ?)",
                )
                .bind(card_id)
                .bind(keyword_id)
                .execute(txn.as_mut())
                .await
                .with_context(|| {
                    format!(
                        "inserting card and keyword - {} {keyword}",
                        card.name.as_ref().unwrap()
                    )
                })?;
            }
        }

        Ok(())
    }

    async fn add_images(
        &self,
        card: &ScryfallCard<'_>,
        txn: &mut SqliteTransaction<'_>,
    ) -> Result<()> {
        let card_id: i64 = sqlx::query_scalar("select id from card where card_id = ?")
            .bind(&card.id)
            .fetch_one(txn.as_mut())
            .await
            .with_context(|| {
                format!(
                    "fetching card id for images - {}",
                    card.name.as_ref().unwrap()
                )
            })?;

        if let Some(ref images) = card.image_uris {
            insert_image!(&card, txn, images, art_crop, card_id);
            insert_image!(&card, txn, images, png, card_id);
            insert_image!(&card, txn, images, normal, card_id);
            insert_image!(&card, txn, images, large, card_id);
            insert_image!(&card, txn, images, small, card_id);
            insert_image!(&card, txn, images, border_crop, card_id);
        }
        Ok(())
    }

    async fn add_card_types(
        &self,
        card: &ScryfallCard<'_>,
        txn: &mut SqliteTransaction<'_>,
    ) -> Result<()> {
        let (supertype, card_types, subtypes) = card.extract_types();

        let card_id: i64 = sqlx::query_scalar("select id from card where card_id = ?")
            .bind(&card.id)
            .fetch_one(txn.as_mut())
            .await
            .with_context(|| {
                format!(
                    "fetching card id (supertype) - {}",
                    card.name.as_ref().unwrap()
                )
            })?;

        if let Some(supertype) = supertype {
            sqlx::query("insert or ignore into card_supertype(card_id, supertype) values(?, ?)")
                .bind(card_id)
                .bind(supertype)
                .execute(txn.as_mut())
                .await
                .with_context(|| {
                    format!(
                        "inserting supertype - {} {}",
                        card.name.as_ref().unwrap(),
                        supertype
                    )
                })?;
        }

        if let Some(card_types) = card_types {
            for ct in card_types {
                sqlx::query("insert or ignore into card_type(card_id, type) values(?, ?)")
                    .bind(card_id)
                    .bind(ct)
                    .execute(txn.as_mut())
                    .await
                    .with_context(|| {
                        format!("inserting type - {} {}", card.name.as_ref().unwrap(), ct)
                    })?;
            }
        }

        if let Some(subtypes) = subtypes {
            for st in subtypes {
                sqlx::query("insert or ignore into card_subtype(card_id, subtype) values(?, ?)")
                    .bind(card_id)
                    .bind(st)
                    .execute(txn.as_mut())
                    .await
                    .with_context(|| {
                        format!("inserting subtype - {} {}", card.name.as_ref().unwrap(), st)
                    })?;
            }
        }

        Ok(())
    }
}
