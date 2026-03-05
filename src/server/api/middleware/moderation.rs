use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    player::{moderation_status::ModerationStatus, traits::FetchForPlayerUuid},
    server::{
        api::extractors::authentication::{Authentication, HeaderAuthentication},
        state::AppState,
    },
};

#[axum::debug_middleware]
pub async fn moderation_middleware(
    State(state): State<Arc<AppState>>,
    HeaderAuthentication(Authentication { uuid, .. }): HeaderAuthentication,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let moderation_status = ModerationStatus::fetch_for_player_uuid(&state, &uuid)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))?;
    if moderation_status.is_banned() {
        return Err((StatusCode::UNAUTHORIZED, "user is banned"));
    }
    request.extensions_mut().insert(uuid);
    // double it and give it to the next person
    Ok(next.run(request).await)
}
