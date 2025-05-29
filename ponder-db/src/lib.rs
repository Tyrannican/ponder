use anyhow::{Context, Result};
use scryfall::ScryfallCard;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

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
        Ok(Self { pool })
    }

    async fn setup_db(pool: &SqlitePool) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .context("running database migrations")?;

        Ok(())
    }

    // Insert card into all tables with insert or ignore
    async fn add_card(&self, card: &scryfall::ScryfallCard<'_>) -> Result<()> {
        sqlx::query("insert or ignore into card() values()")
            .execute(&self.pool)
            .await
            .with_context(|| format!("insert {} into database", card.name))?;
        Ok(())
    }
}
