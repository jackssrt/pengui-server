use std::{
    num::NonZeroU16,
    sync::{Arc, nonpoison::RwLock},
};

use anyhow::{Result, anyhow};
use axum::{
    extract::{Query, Request, State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::instrument;

use crate::{
    client::Client,
    player::Player,
    room::{
        Room,
        client::{RoomClient, packet::OutgoingPacket},
    },
    server::{
        api::extractors::authentication::OptionalQueryAuthentication,
        error::AppError,
        state::{AppState, Rooms},
    },
};

#[derive(Deserialize)]
pub struct RoomQuery {
    id: u16,
}

#[axum::debug_handler]
#[instrument(skip_all)]
pub async fn handle_room(
    ws: WebSocketUpgrade,
    State(state): State<&'static AppState>,
    OptionalQueryAuthentication(auth): OptionalQueryAuthentication,
    Query(RoomQuery { id }): Query<RoomQuery>,
    r: Request,
) -> Result<impl IntoResponse, AppError> {
    let room = NonZeroU16::new(id).map_or_else(
        || todo!(),
        |id| {
            let mut rooms = state.rooms.rooms.lock();
            // TODO get real error message
            // TODO handle case where id is 0
            let room_id = state
                .assets
                .is_valid_map_id(id)
                .ok_or_else(|| anyhow!("invalid room id"))?;
            anyhow::Ok(Rooms::get_by_id(&mut rooms, room_id).clone())
        },
    )?;

    let is_authenticated = auth.is_authenticated();
    let uuid = auth.take_uuid();
    let player = state
        .players
        .get_by_uuid(&uuid)
        .await
        .ok_or_else(|| anyhow!("invalid player, are you connected to the session ws?"))?;
    Ok((ws.protocols(["binary"]).on_upgrade(async move |ws| {
        let _ = handle_connection(state, ws, room, player, is_authenticated).await;
    }),))
}

#[instrument(skip_all, fields(uuid = player.read().uuid.0.as_ref()), name = "room ws")]
async fn handle_connection(
    state: &'static AppState,
    ws: WebSocket,
    room: Arc<RwLock<Room>>,
    player: Arc<RwLock<Player>>,
    is_authenticated: bool,
) -> Result<()> {
    tracing::info!(
        "new room connection by {:?} ({}) to room {:?}",
        player.read().uuid.0,
        player.read().ip,
        room.read().id
    );
    let (client, fut) = RoomClient::new(state, room.clone(), Arc::downgrade(&player), ws);
    let client = Arc::new(client);
    player.write().room_client = Some(client.clone());

    send_sync_packet(&player, is_authenticated, &client).await?;
    client.state.lock().await.join_room().await?;
    fut.await;
    client.state.lock().await.leave_current_room().await?;

    tracing::info!("goodbye");
    Ok(())
}

async fn send_sync_packet(
    player: &RwLock<Player>,
    is_authenticated: bool,
    client: &RoomClient,
) -> Result<()> {
    let key = client.state.lock().await.cryptography.key;
    let packet = {
        let player = player.read();
        OutgoingPacket::Sync {
            id: player.id,
            key,
            uuid: player.uuid.clone(),
            rank: player.rank.clone(),
            is_authenticated,
            badge: player.badge.clone(),
            medals: player.medals.clone(),
        }
    };
    client.send_packet(packet).await?;
    Ok(())
}
