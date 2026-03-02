use std::{os::fd::IntoRawFd, sync::Arc};

use axum::{
    BoxError, Router, ServiceExt,
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    response::IntoResponse,
    routing::{IntoMakeService, get},
    serve::Listener,
};
use tower::{Layer, MakeService, Service, ServiceBuilder, util::MapRequestLayer};

use crate::server::{
    api::{
        middleware::{command_query::rewrite_command_query, moderation::moderation_middleware},
        players::handle_players,
        save_sync::handle_savesync_timestamp,
        session::handle_session,
    },
    state::AppState,
};

mod extractors;
mod middleware;
mod players;
mod save_sync;
mod session;

async fn handle_error(err: impl Into<BoxError>) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", err.into()),
    )
}
pub async fn setup_router(state: Arc<AppState>, listener: impl Listener) {
    let authenticated = Router::new()
        .route("/api/savesync/get", get(handle_savesync_timestamp))
        .route("/api/savesync/list", get(handle_savesync_timestamp))
        .route("/api/savesync/clear", get(handle_savesync_timestamp))
        .route("/api/savesync/push", get(handle_savesync_timestamp))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(from_fn_with_state(
                    Arc::clone(&state),
                    moderation_middleware,
                    // 8 mb
                ))
                .layer(DefaultBodyLimit::max(8 * 1024 * 1024)),
        );
    let command_query_mapper = MapRequestLayer::new(rewrite_command_query);
    println!("Now serving requests.");
    axum::serve(
        listener,
        command_query_mapper
            .layer(
                Router::new()
                    .route("/session", get(handle_session))
                    .route("/players", get(handle_players))
                    .merge(authenticated)
                    .with_state(state),
            )
            .into_make_service(),
    )
    .into_future()
    .await?;
}
