use core::fmt::Debug;
use std::{os::fd::IntoRawFd, sync::Arc};

use anyhow::Result;
use axum::{
    BoxError, Router, ServiceExt,
    error_handling::{HandleError, HandleErrorLayer},
    extract::DefaultBodyLimit,
    http::{Request, StatusCode},
    middleware::{from_fn_with_state, map_request},
    response::IntoResponse,
    routing::{IntoMakeService, get},
    serve::Listener,
};
use tower::{Layer, MakeService, Service, ServiceBuilder};

use crate::server::{
    api::{
        middleware::{command_query::rewrite_command_query, moderation::moderation_middleware},
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
mod players;
mod save_sync;
mod session;

pub async fn setup_router<L>(state: Arc<AppState>, listener: L) -> Result<()>
where
    L: Listener,
    L::Addr: Debug,
{
    let authenticated = Router::new()
        .route("/api/savesync/get", get(handle_savesync_get))
        .route("/api/savesync/timestamp", get(handle_savesync_timestamp))
        .route("/api/savesync/clear", get(handle_savesync_clear))
        .route("/api/savesync/push", get(handle_savesync_push))
        .layer(
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
        .merge(authenticated)
        .with_state(state);
    let app = map_request(rewrite_command_query).layer(app);
    println!("Now serving requests.");
    axum::serve(listener, app.into_make_service())
        .into_future()
        .await?;

    Ok(())
}
