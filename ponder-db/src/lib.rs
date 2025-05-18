mod scryfall;

#[cfg(test)]
mod scryfall_tests {
    use super::*;

    #[tokio::test]
    async fn checker() -> anyhow::Result<()> {
        let cards = scryfall::download().await?;
        Ok(())
    }
}
