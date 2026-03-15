use std::{
    ops::ControlFlow,
    sync::{Arc, nonpoison::RwLock},
};

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use bstr::ByteSlice;
use tokio::{
    select,
    sync::{Mutex, mpsc},
};

use crate::{
    player::Player,
    room::{
        Room,
        client::{
            cryptography::Cryptography,
            packet::{IncomingRoomPacket, OutgoingRoomPacket, error::PacketError},
            state::RoomClientState,
        },
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
    outgoing_sender: mpsc::Sender<OutgoingRoomPacket>,
    pub cryptography: Arc<std::sync::nonpoison::Mutex<Cryptography>>,
}

impl RoomClient {
    pub fn new(
        state: Arc<AppState>,
        room: Arc<RwLock<Room>>,
        player: Arc<RwLock<Player>>,
        mut socket: WebSocket,
    ) -> (impl Future<Output = ()>, Self) {
        let (outgoing_sender, mut outgoing_receiver) = mpsc::channel::<OutgoingRoomPacket>(100);
        let state = RoomClientState::new(state, player, outgoing_sender.clone(), room);
        let crypto = Arc::new(std::sync::nonpoison::Mutex::new(Cryptography::new()));
        #[allow(clippy::unwrap_used)]
        let fut = {
            let crypto = crypto.clone();
            let state = state.clone();
            async move {
                loop {
                    select! {
                        Some(packet) = outgoing_receiver.recv() => {
                            if Self::handle_outgoing(&mut socket, packet).await == ControlFlow::Break(()) {
                                break;
                            }
                        },
                        Some(Ok(Message::Binary(data))) = socket.recv() => {
                            if Self::handle_incoming(&state, &crypto, &data).await == ControlFlow::Break(()) {
                                break;
                            }
                        },
                        else => {tracing::error!("broken connection"); break}
                    }
                }
            }
        };

        (
            fut,
            Self {
                state,
                outgoing_sender,
                cryptography: crypto,
            },
        )
    }
    async fn handle_incoming(
        state: &Mutex<RoomClientState>,
        crypto: &std::sync::nonpoison::Mutex<Cryptography>,
        data: &[u8],
    ) -> ControlFlow<()> {
        let data = {
            let mut crypto = crypto.lock();
            let Some(data) = crypto.verify_bytes(data) else {
                tracing::error!("failed cryptography checks");
                return ControlFlow::Break(());
            };
            data
        };
        // deserialize and do stuff here
        for packet_bytes in data.split_str("\u{FFFE}") {
            let Ok(packet) = IncomingRoomPacket::from_bytes(packet_bytes) else {
                tracing::error!("failed to deserialize");
                return ControlFlow::Break(());
            };
            tracing::trace!("handling packet {:?}", packet);
            if let Err(e) = state.lock().await.handle_incoming_packet(packet).await {
                tracing::error!("failed to handle packet, trace me to find out which packet, {e}");
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
    async fn handle_outgoing(
        socket: &mut WebSocket,
        packet: OutgoingRoomPacket,
    ) -> ControlFlow<()> {
        let Ok(bytes) = (if let OutgoingRoomPacket::Multiple(packets) = packet {
            packets
                .into_iter()
                .map(OutgoingRoomPacket::into_bytes)
                .collect::<Result<Vec<_>, PacketError>>()
                .map(|x| x.join(bstr::B(&[0xff, 0xfe])))
        } else {
            packet.into_bytes()
        }) else {
            tracing::error!("failed to serialize");
            return ControlFlow::Break(());
        };
        let res = socket.send(Message::Binary(bytes.into())).await;
        ControlFlow::Continue(())
    }

    pub async fn send_packet(&self, packet: OutgoingRoomPacket) -> Result<()> {
        self.outgoing_sender.send(packet).await?;
        Ok(())
    }
}
