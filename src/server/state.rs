use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;

use crate::server::{
    args::Args, assets::Assets, config::Config, database::Database, parties::Parties,
    players::Players, rooms::Rooms,
};
pub struct AppState {
    pub args: Args,
    pub config: Arc<Config>,
    pub database: Database,
    pub assets: Assets,
    pub players: Players,
    pub parties: Parties,
    pub rooms: Rooms,
}

impl AppState {
    pub async fn setup() -> Result<Self> {
        // Args
        let args = Args::parse();

        // Config
        let config = Arc::new(
            Config::parse(&args.config)
                .await
                .context("failed to read config")?,
        );

        // Database
        let database = Database::connect(&config)
            .await
            .context("failed to connect to database")?;

        // Assets
        let assets;
        {
            let config = Arc::clone(&config);
            assets = Assets::new(config).context("failed to init assets")?;
        }

        // State
        Ok(Self {
            args,
            config,
            database,
            assets,
            players: Players::default(),
            parties: Parties::default(),
            rooms: Rooms::default(),
        })
    }
}
