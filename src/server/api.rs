use std::{fmt::Debug, sync::Arc, time::Duration};

use anyhow::Result;
use axum::{
    Router, ServiceExt,
    body::Bytes,
    extract::DefaultBodyLimit,
    http::HeaderValue,
    middleware::{from_fn_with_state, map_request},
    routing::{any, get},
    serve::Listener,
};
use tower::Layer;
use tower_http::{
    cors::{self, CorsLayer},
    trace::TraceLayer,
};
use tracing::Span;

use crate::server::{
    api::{
        middleware::{command_query::rewrite_command_query, moderation::moderation_middleware},
        player_info::handle_player_info,
        players::handle_players,
        room::handle_room,
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
mod room;
mod save_sync;
mod session;

// feel free to change :D
static ALLOWED_ORIGINS: &[&str] = &["http://localhost:*", "https://ynoproject.net"];

#[allow(clippy::unwrap_used)]
pub async fn setup_router<L>(state: Arc<AppState>, listener: L) -> Result<()>
where
    L: Listener,
    L::Addr: Debug,
{
    let authenticated = Router::new()
        .route("/api/savesync/get", get(handle_savesync_get))
        .route("/api/savesync/timestamp", get(handle_savesync_timestamp))
        .route("/api/savesync/clear", get(handle_savesync_clear))
        .merge(
            Router::new()
                .route("/api/savesync/push", get(handle_savesync_push))
                .route_layer(
                    DefaultBodyLimit::max(8 * 1024 * 1024), // 8 mb
                ),
        )
        .route_layer(from_fn_with_state(
            Arc::clone(&state),
            moderation_middleware,
        ));
    let websockets = Router::new()
        .route("/session", any(handle_session))
        .route("/room", any(handle_room))
        .route_layer(
            CorsLayer::new()
                .allow_origin(cors::Any)
                .allow_methods(cors::Any),
        );
    let app = Router::new()
        .route("/players", get(handle_players))
        .route("/api/info", get(handle_player_info))
        .merge(websockets)
        .merge(authenticated)
        .layer(TraceLayer::new_for_http().on_body_chunk(
            |chunk: &Bytes, latency: Duration, _span: &Span| {
                tracing::debug!("sending {:?}", chunk);
            },
        ))
        .route_layer(
            CorsLayer::new()
                .allow_origin(
                    ALLOWED_ORIGINS
                        .iter()
                        .map(|origin| origin.parse::<HeaderValue>().unwrap())
                        .collect::<Vec<_>>(),
                )
                .allow_credentials(true),
        )
        .with_state(state);
    let app = map_request(rewrite_command_query).layer(app);
    tracing::info!("serving requests");
    axum::serve(listener, app.into_make_service())
        .into_future()
        .await?;

    Ok(())
}
