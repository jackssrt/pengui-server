use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    player::{ids::PlayerUuid, moderation_status::ModerationStatus, traits::FetchForPlayerUuid},
    server::{api::extractors::token::AuthorizationToken, state::AppState},
};

#[axum::debug_middleware]
pub async fn moderation_middleware(
    State(state): State<Arc<AppState>>,
    AuthorizationToken(token): AuthorizationToken,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let uuid = PlayerUuid::fetch_for_token(&state, &token)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))?
        .ok_or((StatusCode::UNAUTHORIZED, ""))?;
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
