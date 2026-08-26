use std::{fs::create_dir, os::unix::fs::PermissionsExt, path::PathBuf, process, time::Duration};

use anyhow::{Context as _, Result};
use futures_util::StreamExt;
use tokio::net::UnixListener;

use crate::{
    chat::history::init_history,
    server::state::{AppState, config::Config},
    session::init_session,
};

pub mod api;
pub mod error;
pub mod saves;
pub mod state;

pub fn get_listener(config: &Config) -> Result<UnixListener> {
    let socket_path = PathBuf::from(format!("sockets/{}.sock", config.game_name));
    // okay because socket path will always have a parent
    // because we defined it
    // throw away the Err, because it's most likely just because it already exists
    #[allow(clippy::unwrap_used)]
    if let Err(reason) = create_dir(socket_path.parent().unwrap())
        && reason.kind() != std::io::ErrorKind::AlreadyExists
    {
        return Err(reason).context("failed to create unix socket parent directory");
    }

    // delete the old socket
    if let Err(reason) = std::fs::remove_file(&socket_path)
        && reason.kind() != std::io::ErrorKind::NotFound
    {
        return Err(reason).context("failed to delete unix socket");
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
fn setup_shutdown_handler(state: &'static AppState) -> Result<()> {
    tokio::spawn(
        signal_hook_tokio::Signals::new([signal_hook::consts::SIGTERM])?.for_each(async |_| {
            state
                .players
                .broadcast_system_message("**The server is restarting.**".into())
                .await;

            tokio::time::sleep(Duration::from_secs(1)).await;
            process::exit(0);
        }),
    );
    Ok(())
}

pub async fn start() -> Result<()> {
    tracing::info!("starting");
    let state = Box::leak(Box::new(AppState::setup().await?));

    setup_shutdown_handler(state)?;

    init_history(state);
    init_session(state);

    setup_router(state).await?;

    Ok(())
}
