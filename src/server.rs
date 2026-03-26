use std::{fs::create_dir, os::unix::fs::PermissionsExt, path::PathBuf};

use anyhow::Result;
use tokio::net::UnixListener;

use crate::{
    chat::history::init_history,
    server::{config::Config, state::AppState},
    session::init_session,
};

pub mod api;
pub mod args;
pub mod assets;
pub mod config;
pub mod database;
pub mod error;
pub mod parties;
pub mod players;
pub mod rooms;
pub mod saves;
pub mod state;

pub fn get_listener(config: &Config) -> Result<UnixListener> {
    let socket_path = PathBuf::from(format!("sockets/{}.sock", config.game_name));
    // okay because socket path will always have a parent
    // because we defined it
    // throw away the Err, because it's most likely just because it already exists
    #[allow(clippy::unwrap_used)]
    let _ = create_dir(socket_path.parent().unwrap());

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
async fn setup_router(state: &'static AppState) -> Result<()> {
    // listen
    let listener = get_listener(&state.config)?;
    api::setup_router(state, listener).await?;

    Ok(())
}

pub async fn start() -> Result<()> {
    tracing::info!("starting");
    let state = Box::leak(Box::new(AppState::setup().await?));

    init_history(state);
    init_session(state);

    setup_router(state).await?;

    Ok(())
}
