use axum::extract::State;

use crate::server::state::AppState;

#[axum::debug_handler]
pub async fn handle_players(State(state): State<&'static AppState>) -> String {
    state.players.players.lock().len().to_string()
}
