use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::server::{api::extractors::token::AuthorizationToken, state::AppState};

#[axum::debug_middleware]
pub async fn moderation_middleware(
    State(state): State<Arc<AppState>>,
    AuthorizationToken(token): AuthorizationToken,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let (uuid, moderation_status) = state
        .database
        .get_player_data_for_token(&token)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, ""))?
        .ok_or((StatusCode::UNAUTHORIZED, ""))?;
    if moderation_status.is_banned() {
        return Err((StatusCode::UNAUTHORIZED, "user is banned"));
    }
    request.extensions_mut().insert(uuid);
    // double it and give it to the next person
    Ok(next.run(request).await)
}
