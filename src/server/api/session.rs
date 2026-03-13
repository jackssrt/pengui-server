use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::{ConnectInfo, State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
};

use anyhow::Result;
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
            eprintln!("session handler error {e:?}");
        }
    }))
}

async fn handle_session_websocket(
    socket: WebSocket,
    state: Arc<AppState>,
    auth: OptionalAuthentication,
    ip: IpAddr,
) -> Result<()> {
    let player = Player::new(
        &state,
        SessionClient::new(state.clone(), socket).await?,
        auth.is_authenticated(),
        auth.take_uuid(),
        ip,
    )
    .await?;
    Ok(())
}
