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
mod updater;

use updater::DatabaseUpdater;

#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
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
        DatabaseUpdater::new(&self.pool).update().await
    }
}
