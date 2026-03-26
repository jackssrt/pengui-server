use std::net::IpAddr;

use anyhow::Result;
use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
};
use axum_client_ip::RightmostXForwardedFor;
use tokio::sync::mpsc;
use tracing::instrument;

use crate::{
    player::{Player, ids::PlayerUuid},
    server::{
        api::extractors::authentication::OptionalQueryAuthentication, error::AppError,
        state::AppState,
    },
};

#[axum::debug_handler]
pub async fn handle_session(
    State(state): State<&'static AppState>,
    ws: WebSocketUpgrade,
    OptionalQueryAuthentication(auth): OptionalQueryAuthentication,
    RightmostXForwardedFor(ip): RightmostXForwardedFor,
) -> Result<Response, AppError> {
    Ok(ws.on_upgrade(async move |socket| {
        if let Err(e) =
            handle_connection(state, auth.is_authenticated(), auth.take_uuid(), ip, socket).await
        {
            tracing::error!("session handler error {e:?}");
        }
    }))
}
#[instrument(skip_all, fields(uuid = uuid.0), name = "session ws")]
async fn handle_connection(
    state: &'static AppState,
    is_authenticated: bool,
    uuid: PlayerUuid,
    ip: IpAddr,
    socket: WebSocket,
) -> Result<()> {
    let (outgoing_sender, outgoing_receiver) = mpsc::channel(100);
    let player = Player::new(state, is_authenticated, uuid, ip, outgoing_sender).await?;
    player
        .with(|player| player.session_client.clone())
        .run(socket, outgoing_receiver)
        .await;

    state.players.remove_player(player);
    Ok(())
}
