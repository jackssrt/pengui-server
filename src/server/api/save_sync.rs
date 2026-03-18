use std::sync::Arc;

use axum::{
    Extension,
    body::Body,
    extract::{Request, State},
    response::{IntoResponse, Response},
};
use futures_util::TryStreamExt;
use tokio_util::io::{ReaderStream, StreamReader};

use crate::{
    player::ids::PlayerUuid,
    server::{
        error::AppError,
        saves::{
            clear_game_save_data, create_game_save_data, get_save_data, get_save_data_timestamp,
        },
        state::AppState,
    },
};

#[axum::debug_handler]
pub async fn handle_savesync_timestamp(
    State(state): State<Arc<AppState>>,
    Extension(player_uuid): Extension<PlayerUuid>,
    r: Request,
) -> Result<String, AppError> {
    Ok(get_save_data_timestamp(&state, &player_uuid)
        .await?
        .to_rfc3339())
}

#[axum::debug_handler]
pub async fn handle_savesync_get(
    State(state): State<Arc<AppState>>,
    Extension(player_uuid): Extension<PlayerUuid>,
    r: Request,
) -> Result<impl IntoResponse, AppError> {
    Ok(
        Response::builder().body(Body::from_stream(ReaderStream::new(
            get_save_data(&state, &player_uuid).await?,
        )))?,
    )
}

#[axum::debug_handler]
pub async fn handle_savesync_push(
    State(state): State<Arc<AppState>>,
    Extension(player_uuid): Extension<PlayerUuid>,
    r: Request,
) -> Result<impl IntoResponse, AppError> {
    // the size of this stream is already limited by
    // the middleware setup in the router
    let data_stream = r
        .into_body()
        .into_data_stream()
        .map_err(std::io::Error::other);
    let data_reader = StreamReader::new(data_stream);

    create_game_save_data(&state, &player_uuid, data_reader).await?;
    Ok(())
}

#[axum::debug_handler]
pub async fn handle_savesync_clear(
    State(state): State<Arc<AppState>>,
    Extension(player_uuid): Extension<PlayerUuid>,
) -> Result<impl IntoResponse, AppError> {
    clear_game_save_data(&state, &player_uuid).await?;
    Ok(())
}
