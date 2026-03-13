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

use crate::{
    player::Player,
    room::{
        Room,
        client::{RoomClient, packet::OutgoingRoomPacket},
    },
    server::{
        api::extractors::authentication::OptionalQueryAuthentication, error::AppError,
        rooms::Rooms, state::AppState,
    },
};

#[derive(Deserialize)]
pub struct RoomQuery {
    id: u16,
}

#[axum::debug_handler]
pub async fn handle_room(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    OptionalQueryAuthentication(auth): OptionalQueryAuthentication,
    Query(RoomQuery { id }): Query<RoomQuery>,
    r: Request,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("new room client");
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

    tracing::info!("got room");
    let is_authenticated = auth.is_authenticated();
    let uuid = auth.take_uuid();
    tracing::info!("uuid");
    let player = state
        .players
        .get_by_uuid(&uuid)
        .await
        .ok_or_else(|| anyhow!("invalid player, are you connected to the session ws?"))?;
    tracing::info!("plr");
    Ok(ws.on_upgrade(async move |ws| {
        if let Err(err) = handle_connection(state, ws, room, player, is_authenticated).await {
            eprintln!("error in room websocket: {err}");
        }
    }))
}

async fn handle_connection(
    state: Arc<AppState>,
    ws: WebSocket,
    room: Arc<RwLock<Room>>,
    player: Arc<RwLock<Player>>,
    is_authenticated: bool,
) -> Result<()> {
    tracing::info!(
        "new room connection by {:?} to room {:?}",
        player.read().uuid.0,
        room.read().id
    );
    let (fut, client) = RoomClient::new(state, room.clone(), player.clone(), ws);
    room.write().players.push(player.clone());
    let packet = {
        let player = player.read();
        OutgoingRoomPacket::Sync {
            id: player.id,
            key: client.cryptography.key,
            uuid: player.uuid.clone(),
            rank: player.rank.clone(),
            is_authenticated,
            badge: player.badge.clone(),
            medals: player.medals.clone(),
        }
    };
    client.send_packet(packet).await?;
    let id = room.read().id.clone();
    client.send_packet(OutgoingRoomPacket::RoomId(id)).await?;
    fut.await;
    tracing::error!("aa byee");
    Ok(())
}
