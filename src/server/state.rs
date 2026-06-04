use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::server::state::{
    args::Args, assets::Assets, config::Config, database::Database, parties::Parties,
    players::Players, rooms::Rooms,
};

pub mod args;
pub mod assets;
pub mod config;
pub mod database;
pub mod parties;
pub mod players;
pub mod rooms;

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
        let config = Arc::new(Config::parse(&args.config).context("failed to read config")?);

        // Logging
        let (file_logger, _guard) = tracing_appender::non_blocking(
            #[allow(clippy::expect_used)]
            tracing_appender::rolling::Builder::new()
                .filename_prefix("pengui-server")
                .filename_suffix(".log")
                .build(format!("logs/{}", config.game_name))
                .expect("failed to build logger"),
        );
        // disable timestamps in debug mode
        #[cfg(debug_assertions)]
        let stdout_logger = tracing_subscriber::fmt::layer().without_time();
        #[cfg(not(debug_assertions))]
        let stdout_logger = tracing_subscriber::fmt::layer();

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    // axum logs rejections from built-in extractors with the `axum::rejection`
                    // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                    format!(
                        "{}=info,tower_http=debug,axum::rejection=trace",
                        env!("CARGO_CRATE_NAME")
                    )
                    .into()
                }),
            )
            .with(stdout_logger)
            .with(tracing_subscriber::fmt::layer().with_writer(file_logger))
            .init();

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
