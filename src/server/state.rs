use std::sync::Arc;

use anyhow::Result;
use clap::Parser;

use crate::server::{
    args::Args, assets::Assets, config::Config, database::Database, parties::Parties,
    players::Players,
};
pub struct AppState {
    pub args: Args,
    pub config: Arc<Config>,
    pub database: Database,
    pub assets: Assets,
    pub players: Players,
    pub parties: Parties,
}

impl AppState {
    pub async fn setup() -> Result<AppState> {
        // Args
        let args = Args::parse();

        // Config
        let config = Arc::new(Config::parse(&args.config).await?);

        // Database
        let database = Database::connect(&config).await?;

        // Assets
        let assets;
        {
            let config = Arc::clone(&config);
            assets = Assets::new(config);
        }

        // Players
        let players = Players::default();

        // Parties
        let parties = Parties::default();

        // State
        Ok(AppState {
            args,
            config,
            database,
            assets,
            players,
            parties,
        })
    }
}
