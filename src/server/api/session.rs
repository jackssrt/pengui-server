use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::{ConnectInfo, State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
};

use anyhow::Result;

use crate::{
    player::Player,
    server::{
        api::extractors::authentication::{
            OptionalAuthentication, OptionalQueryAuthentication,
        },
        error::AppError,
        state::AppState,
    },
    session::client::SessionClient,
};

#[axum::debug_handler]
pub async fn handle_session(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    OptionalQueryAuthentication(auth): OptionalQueryAuthentication,
) -> Result<Response, AppError> {
    let ip = socket_addr.ip();

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
        SessionClient::new(socket),
        auth.is_authenticated(),
        auth.take_uuid(),
        ip,
    )
    .await?;
    Ok(())
}
