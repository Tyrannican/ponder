use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

use std::{path::PathBuf, str::FromStr};

mod scryfall;

#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub(crate) async fn load(ws: PathBuf) -> Result<Self> {
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
}
