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
    server::{error::AppError, saves, state::AppState},
};

#[axum::debug_handler]
pub async fn handle_savesync_timestamp(
    State(state): State<&'static AppState>,
    Extension(player_uuid): Extension<PlayerUuid>,
    r: Request,
) -> Result<String, AppError> {
    Ok(saves::get_timestamp(state, &player_uuid)
        .await?
        .to_rfc3339())
}

#[axum::debug_handler]
pub async fn handle_savesync_get(
    State(state): State<&'static AppState>,
    Extension(player_uuid): Extension<PlayerUuid>,
    r: Request,
) -> Result<impl IntoResponse, AppError> {
    Ok(
        Response::builder().body(Body::from_stream(ReaderStream::new(
            saves::get(state, &player_uuid).await?,
        )))?,
    )
}

#[axum::debug_handler]
pub async fn handle_savesync_push(
    State(state): State<&'static AppState>,
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

    saves::create(state, &player_uuid, data_reader).await?;
    Ok(())
}

#[axum::debug_handler]
pub async fn handle_savesync_clear(
    State(state): State<&'static AppState>,
    Extension(player_uuid): Extension<PlayerUuid>,
) -> Result<impl IntoResponse, AppError> {
    saves::clear(state, &player_uuid).await?;
    Ok(())
}
