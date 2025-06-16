use std::path::PathBuf;

use anyhow::{Context, Result};
use dotstore;
use ponder_db::SqliteStore;

mod tui;
use tui::Tui;

#[derive(Debug)]
pub struct Ponder {
    pub workspace: PathBuf,
    pub store: SqliteStore,
}

impl Ponder {
    pub async fn new() -> Result<Self> {
        let workspace = dotstore::home_store("ponder")
            .with_context(|| "finding home directory".to_string())?
            .with_context(|| "home directory not set")?;

        Ok(Self {
            store: SqliteStore::load(&workspace).await?,
            workspace,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let ponder = Ponder::new().await?;
    let mut tui = Tui::new(ponder);
    if let Err(e) = tui.run().await {
        drop(tui);
        eprintln!("{e:#?}");
    }

    Ok(())
}
