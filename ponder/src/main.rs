use std::path::PathBuf;

use anyhow::Result;
use dotstore;
use ponder_db::SqliteStore;

#[derive(Debug)]
pub struct Ponder {
    pub workspace: PathBuf,
    pub db: SqliteStore,
}

impl Ponder {
    pub async fn new() -> Result<Self> {
        let workspace = dotstore::home_store("ponder")?.expect("could not find home directory");
        Ok(Self {
            db: SqliteStore::load(&workspace).await?,
            workspace,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Ponder::new().await?;

    Ok(())
}
