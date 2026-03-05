use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router, ServiceExt,
    extract::DefaultBodyLimit,
    middleware::{from_fn_with_state, map_request},
    routing::get,
};
use tokio::net::UnixListener;
use tower::{Layer, ServiceBuilder};

use crate::server::{
    api::{
        middleware::{command_query::rewrite_command_query, moderation::moderation_middleware},
        player_info::handle_player_info,
        players::handle_players,
        save_sync::{
            handle_savesync_clear, handle_savesync_get, handle_savesync_push,
            handle_savesync_timestamp,
        },
        session::handle_session,
    },
    state::AppState,
};

mod extractors;
mod middleware;
mod player_info;
mod players;
mod save_sync;
mod session;

pub async fn setup_router(state: Arc<AppState>, listener: UnixListener) -> Result<()> {
    let authenticated = Router::new()
        .route("/api/savesync/get", get(handle_savesync_get))
        .route("/api/savesync/timestamp", get(handle_savesync_timestamp))
        .route("/api/savesync/clear", get(handle_savesync_clear))
        .route("/api/savesync/push", get(handle_savesync_push))
        .route_layer(
            ServiceBuilder::new()
                .layer(from_fn_with_state(
                    Arc::clone(&state),
                    moderation_middleware,
                ))
                .layer(DefaultBodyLimit::max(8 * 1024 * 1024)), // 8 mb
        );
    let app = Router::new()
        .route("/session", get(handle_session))
        .route("/players", get(handle_players))
        .route("/api/info", get(handle_player_info))
        .merge(authenticated)
        .with_state(state);
    let app = map_request(rewrite_command_query).layer(app);
    println!("Now serving requests.");
    axum::serve(listener, app.into_make_service())
        .into_future()
        .await?;

    Ok(())
}
