use std::path::PathBuf;

use color_eyre::Result;
use dotstore;
use ponder_db::SqliteStore;

#[derive(Debug)]
pub struct AppState {
    workspace: PathBuf,
    db: SqliteStore,
}

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}
