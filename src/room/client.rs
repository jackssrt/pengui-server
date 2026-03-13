use std::sync::{Arc, nonpoison::RwLock};

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures_util::{SinkExt, StreamExt};
use tokio::{
    select, spawn,
    sync::{Mutex, mpsc},
};

use crate::{
    player::Player,
    room::{
        Room,
        client::{
            cryptography::Cryptography,
            packet::{IncomingRoomPacket, OutgoingRoomPacket},
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
    pub cryptography: Arc<Cryptography>,
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
        let crypto = Arc::new(Cryptography::new());
        #[allow(clippy::unwrap_used)]
        let fut = {
            let crypto = crypto.clone();
            let state = state.clone();
            async move {
                tracing::info!("starting room ws handler");
                loop {
                    tracing::info!("hi im in the loop");
                    select! {
                        Some(packet) = outgoing_receiver.recv() => {
                            tracing::info!("send {:?} packet", &packet);
                            let Ok(bytes) = packet
                                .into_bytes()
                                .inspect_err(|x| eprintln!("failed to send room packet: {x}")) else {
                                    tracing::error!("failed to encodee");
                                    break
                                };
                            let Ok(()) = socket.send(Message::Binary(bytes.into())).await else {
                                tracing::error!("failed to send bytes");
                                break
                            };
                        },
                        Some(Ok(Message::Binary(data))) = socket.recv() => {
                            tracing::info!("recving {:?} bytes", &data);
                            let Some(data) = crypto.verify_bytes(&data) else {
                                tracing::error!("failed crypto");
                                break;
                            };
                            // deserialize and do stuff here
                            let packet = IncomingRoomPacket::from_bytes(data).unwrap();
                            state.lock().await.handle_incoming_packet(packet).await.unwrap();
                        },
                        else => {tracing::error!("broken connection"); break}
                    }
                    tracing::error!("looping around");
                }
                tracing::error!("room ws left");
            }
        };

        (
            fut,
            Self {
                cryptography: crypto,
                state,
                outgoing_sender,
            },
        )
    }
    pub async fn send_packet(&self, packet: OutgoingRoomPacket) -> Result<()> {
        self.outgoing_sender.send(packet).await?;
        Ok(())
    }
}
