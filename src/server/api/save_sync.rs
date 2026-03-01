
use axum::response::IntoResponse;

use crate::server::state::AppState;

use std::sync::Arc;

use axum::extract::State;

#[axum::debug_handler]
pub async fn handle_savesync(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.players.players.lock().len().to_string()
}
