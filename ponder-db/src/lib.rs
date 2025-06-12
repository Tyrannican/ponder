use anyhow::{Context, Result};
use scryfall::{Format, ScryfallCard};
use sqlx::{
    Row, SqliteTransaction,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions},
};

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

mod card;
mod scryfall;

#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

macro_rules! insert_image {
    ($card:expr, $txn:expr, $images:expr, $field:ident) => {
        if let Some(ref uri) = $images.$field {
            let img_id: i64 = sqlx::query_scalar("select id from image_type where name = ?")
                .bind(stringify!($field))
                .fetch_one($txn.as_mut())
                .await
                .with_context(|| format!("fetching id for {} image_type", stringify!($field)))?;

            sqlx::query("insert into images(card_id,image_type_id,uri) values(?, ?, ?)")
                .bind(&$card.id)
                .bind(img_id)
                .bind(&uri)
                .execute($txn.as_mut())
                .await
                .with_context(|| format!("inserting image uri - {uri} {}", $card.name))?;
        }
    };
}

// TODO: Move the Update logic to new struct (e.g. SqliteUpdater)
// TODO: Fix Colors and that are arrays so they're numbers in the DB
// TODO: Add logic for Supertypes / Types / Subtypes
// TODO: Deal with Card faces
impl SqliteStore {
    pub async fn load(ws: impl AsRef<Path>) -> Result<Self> {
        let lead = PathBuf::from("sqlite:/");
        let db_name = lead.join(ws).join("ponder.db");

        let connect_opts = SqliteConnectOptions::from_str(db_name.to_str().unwrap())?
            .optimize_on_close(true, None)
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs_f64(0.1))
            .connect_with(connect_opts)
            .await
            .context("creating database")?;

        Self::setup_db(&pool).await?;
        Ok(Self { pool })
    }

    async fn setup_db(pool: &SqlitePool) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .context("running database migrations")?;

        Ok(())
    }

    pub async fn update(&self) -> Result<()> {
        println!("Updating Database...");
        let cards = scryfall::download_latest().await?;
        let mut txn = self.pool.begin().await?;

        self.add_formats(&mut txn).await?;
        self.add_image_types(&mut txn).await?;

        for card in cards.iter() {
            // TODO: Deal with card faces
            if card.card_faces.is_some() {
                continue;
            }
            self.add_card(&card, &mut txn).await?;
            self.add_legalities(&card, &mut txn).await?;
            self.add_keywords(&card, &mut txn).await?;
            self.add_images(&card, &mut txn).await?;
        }

        txn.commit().await?;

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
            sqlx::query("insert into image_type(name) values(?)")
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
                id,
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
            .bind(card_vec_field_to_string!(card, color_indicator))
            .bind(card_vec_field_to_string!(card, produced_mana))
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
            .bind(card_vec_field_to_string!(card, colors))
            .bind(card_vec_field_to_string!(card, color_indicator))
            .bind(&card.rarity)
            .bind(&card.power)
            .bind(&card.toughness)
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
            .with_context(|| format!("insert {} into database", card.name))?;

        Ok(())
    }

    async fn add_legalities(
        &self,
        card: &ScryfallCard<'_>,
        txn: &mut SqliteTransaction<'_>,
    ) -> Result<()> {
        if let Some(legalities) = &card.legalities {
            for (format, legality) in legalities.iter() {
                let format_id: i64 = sqlx::query_scalar("select id from format where name = ?")
                    .bind(&format.to_string())
                    .fetch_one(txn.as_mut())
                    .await?;

                sqlx::query(
                    "insert or ignore into legality(card_id, format_id, status) values(?, ?, ?)",
                )
                .bind(&card.id)
                .bind(&format_id)
                .bind(&legality.to_string())
                .execute(txn.as_mut())
                .await
                .with_context(|| {
                    format!(
                        "inserting legality {:?} ({}) {} ({}) {}\n{card:?}",
                        card.id, card.name, format_id, format, legality
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
                .bind(&card.id)
                .bind(keyword_id)
                .execute(txn.as_mut())
                .await
                .with_context(|| format!("inserting card and keyword - {} {keyword}", card.name))?;
            }
        }

        Ok(())
    }

    async fn add_images(
        &self,
        card: &ScryfallCard<'_>,
        txn: &mut SqliteTransaction<'_>,
    ) -> Result<()> {
        if let Some(ref images) = card.image_uris {
            insert_image!(&card, txn, images, art_crop);
            insert_image!(&card, txn, images, png);
            insert_image!(&card, txn, images, normal);
            insert_image!(&card, txn, images, large);
            insert_image!(&card, txn, images, small);
            insert_image!(&card, txn, images, border_crop);
        }
        Ok(())
    }
}
