use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::Arc};

use anyhow::Result;
use tokio::net::UnixListener;

use crate::{
    chat::history::init_history,
    server::{config::Config, state::AppState},
};

pub mod api;
pub mod args;
pub mod assets;
pub mod config;
pub mod database;
pub mod error;
pub mod parties;
pub mod players;
pub mod saves;
pub mod state;

pub fn get_listener(config: &Config) -> Result<UnixListener> {
    let socket_path = PathBuf::from(format!("sockets/{}.sock", config.game_name));

    // delete the old socket
    if std::fs::exists(&socket_path)? {
        std::fs::remove_file(&socket_path)?;
    }

    // bind the listener
    let listener = UnixListener::bind(&socket_path)?;

    // set the socket permissions
    std::fs::set_permissions(&socket_path, PermissionsExt::from_mode(0o666))?;

    Ok(listener)
}
async fn setup_router(state: Arc<AppState>) -> Result<()> {
    // listen
    let listener = get_listener(&state.config)?;
    api::setup_router(state, listener).await?;

    Ok(())
}

pub async fn start() -> Result<()> {
    println!("Now starting pengui-server...");
    let state = Arc::new(AppState::setup().await?);

    init_history(&state);

    setup_router(state).await?;

    Ok(())
}
