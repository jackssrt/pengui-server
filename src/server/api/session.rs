use std::{net::IpAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
};
use axum_client_ip::RightmostXForwardedFor;

use crate::{
    player::Player,
    server::{
        api::extractors::authentication::{OptionalAuthentication, OptionalQueryAuthentication},
        error::AppError,
        state::AppState,
    },
    session::client::SessionClient,
};

#[axum::debug_handler]
pub async fn handle_session(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    OptionalQueryAuthentication(auth): OptionalQueryAuthentication,
    RightmostXForwardedFor(ip): RightmostXForwardedFor,
) -> Result<Response, AppError> {
    Ok(ws.on_upgrade(async move |socket| {
        if let Err(e) = handle_session_websocket(socket, state, auth, ip).await {
            tracing::error!("session handler error {e:?}");
        }
    }))
}

async fn handle_session_websocket(
    socket: WebSocket,
    state: Arc<AppState>,
    auth: OptionalAuthentication,
    ip: IpAddr,
) -> Result<()> {
    let is_authenticated = auth.is_authenticated();
    let uuid = auth.take_uuid();
    let (session_client, fut) = SessionClient::new(state.clone(), uuid.clone(), socket);
    let player = Player::new(&state, session_client, is_authenticated, uuid, ip).await?;
    fut.await;
    Ok(())
}
