use std::sync::Arc;

use axum::{Router, routing::get};

use crate::server::{
    api::{players::handle_players, save_sync::handle_savesync, session::handle_session},
    state::AppState,
};

mod extractors;
mod players;
mod save_sync;
mod session;

pub fn setup_router(state: &Arc<AppState>) -> Router<()> {
    Router::new()
        .route("/session", get(handle_session))
        .route("/players", get(handle_players))
        .route("/api/savesync", get(handle_savesync))
        .with_state(state.clone())
}
