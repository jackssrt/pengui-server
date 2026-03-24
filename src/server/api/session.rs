use std::{net::IpAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
};
use axum_client_ip::RightmostXForwardedFor;
use tracing::instrument;

use crate::{
    player::{Player, ids::PlayerUuid},
    server::{
        api::extractors::authentication::OptionalQueryAuthentication, error::AppError,
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
        if let Err(e) =
            handle_connection(socket, state, auth.is_authenticated(), auth.take_uuid(), ip).await
        {
            tracing::error!("session handler error {e:?}");
        }
    }))
}
#[instrument(skip_all, fields(uuid = uuid.0), name = "session ws")]
async fn handle_connection(
    socket: WebSocket,
    state: Arc<AppState>,
    is_authenticated: bool,
    uuid: PlayerUuid,
    ip: IpAddr,
) -> Result<()> {
    let (session_client, fut) = SessionClient::new(state.clone(), uuid.clone(), socket);
    let player = Player::new(&state, session_client, is_authenticated, uuid, ip).await?;
    fut.await;
    Ok(())
}
