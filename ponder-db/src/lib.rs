use anyhow::{Context, Result};
use card::Card;
use scryfall::ScryfallCard;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

pub mod card;
mod scryfall;
mod updater;

use updater::DatabaseUpdater;

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
        DatabaseUpdater::new(&self.pool).update().await
    }

    pub async fn query_card_by_name<'a>(&self, name: &str) -> Result<Vec<Card>> {
        let test = format!("%{name}%");
        let results: Vec<Card> = sqlx::query_as("select distinct * from card where name like ?")
            .bind(&test)
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("fetching card by name - {name}"))?;

        Ok(results)
    }
}
