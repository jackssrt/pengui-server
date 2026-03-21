use std::sync::{Arc, nonpoison::RwLock};

use anyhow::{Context, Result, anyhow};
use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use bstr::ByteSlice;
use tokio::sync::{Mutex, mpsc};

use super::Room;
use crate::{
    client::{Client, packet::error::PacketError, state::ClientState},
    player::Player,
    room::client::{
        packet::{IncomingPacket, OutgoingPacket},
        state::RoomClientState,
    },
    server::state::AppState,
};

pub mod cryptography;
pub mod direction;
pub mod flash;
pub mod packet;
pub mod state;

pub struct RoomClient {
    pub state: Arc<Mutex<RoomClientState>>,
    outgoing_sender: mpsc::Sender<OutgoingPacket>,
}
impl RoomClient {
    pub fn new(
        app_state: Arc<AppState>,
        room: Arc<RwLock<Room>>,
        player: Arc<RwLock<Player>>,
        socket: WebSocket,
    ) -> (Self, impl Future<Output = ()>) {
        let (sender, recv) = mpsc::channel(16);
        let state = RoomClientState::new(app_state, room, player, sender.clone());
        let fut = Self::run(socket, state.clone(), recv);

        (
            Self {
                state,
                outgoing_sender: sender,
            },
            fut,
        )
    }
}

impl Client for RoomClient {
    type OutgoingPacket = OutgoingPacket;
    type State = RoomClientState;
    async fn handle_incoming(state: &Mutex<Self::State>, message: Message) -> Result<()> {
        let Message::Binary(data) = message else {
            return Ok(());
        };
        let data = {
            let crypto = &mut state.lock().await.cryptography;

            crypto
                .verify_bytes(&data)
                .ok_or_else(|| anyhow!("failed cryptography checks"))?
        };
        for packet_bytes in data.split_str("\u{FFFE}") {
            let packet = IncomingPacket::from_bytes(packet_bytes)
                .map_err(|_| anyhow!("failed to deserialize"))?;
            tracing::trace!("handling packet {:?}", packet);
            state
                .lock()
                .await
                .process_packet(packet)
                .await
                .context("failed to handle packet, trace me to find out which")?;
        }
        Ok(())
    }
    async fn handle_outgoing(
        state: &Mutex<Self::State>,
        socket: &mut WebSocket,
        packet: OutgoingPacket,
    ) -> Result<()> {
        let bytes = (if let OutgoingPacket::Multiple(packets) = packet {
            packets
                .into_iter()
                .map(OutgoingPacket::into_bytes)
                .collect::<Result<Vec<_>, PacketError>>()
                .map(|x| x.join(bstr::B("\u{FFFE}")).into())
        } else {
            packet.into_bytes()
        })
        .context("failed to serialize")?;
        socket
            .send(Message::Binary(Bytes::from_owner(bytes)))
            .await
            .context("failed to send packet")?;
        Ok(())
    }

    async fn send_packet(&self, packet: OutgoingPacket) -> Result<()> {
        self.outgoing_sender.send(packet).await?;
        Ok(())
    }
    async fn broadcast(&self, packet: Self::OutgoingPacket) -> Result<()> {
        self.state.lock().await.broadcast(packet).await
    }
}
