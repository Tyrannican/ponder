use anyhow::{Context, Result};
use card::Card;
use scryfall::ScryfallCard;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
};

mod card;
mod scryfall;

// create table if not exists legalities (
//     standard boolean,
//     pioneer boolean,
//     modern boolean,
//     premodern boolean,
//     legacy boolean,
//     vintage boolean,
//     commander boolean,
//     pauper boolean,
//     paupercommander boolean,
//     penny boolean,
//     historic boolean,
//     predh boolean,
//     future boolean,
//     alchemy boolean,
//     oathbreaker boolean,
//     timeless boolean,
//     gladiator boolean,
//     standardbrawl boolean,
//     duel boolean,
//     oldschool boolean,
//     explorer boolean,
//     brawl boolean,
// );
#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

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
        let cards = scryfall::download_latest().await?;
        Self::init(cards);
        Ok(Self { pool })
    }

    async fn setup_db(pool: &SqlitePool) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .context("running database migrations")?;

        Ok(())
    }

    fn init(cards: Vec<ScryfallCard>) {
        let mut hs = HashSet::new();
        for card in cards.into_iter() {
            Card::from_scryfall(card, &mut hs);
        }

        for item in hs.iter() {
            println!("{item}");
        }
    }
}
