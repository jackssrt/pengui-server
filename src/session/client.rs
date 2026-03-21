use std::sync::Arc;

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use bstr::BStr;
use tokio::sync::{Mutex, mpsc};

use crate::{
    client::{Client, state::ClientState},
    player::ids::PlayerUuid,
    server::state::AppState,
    session::client::{
        packet::{IncomingPacket, OutgoingPacket},
        state::SessionState,
    },
};

pub mod packet;
pub mod state;

pub struct SessionClient {
    state: Arc<Mutex<SessionState>>,
}
impl Client for SessionClient {
    type OutgoingPacket = OutgoingPacket;
    type State = SessionState;
    async fn handle_incoming(state: &Mutex<Self::State>, message: Message) -> Result<()> {
        let Message::Text(data) = message else {
            return Ok(());
        };
        let packet = IncomingPacket::from_bstr(BStr::new(data.as_bytes()))?;
        state.lock().await.process_packet(packet).await?;
        Ok(())
    }
    async fn handle_outgoing(
        state: &Mutex<Self::State>,
        socket: &mut WebSocket,
        packet: Self::OutgoingPacket,
    ) -> Result<()> {
        let data = packet.into_bstring()?;
        let data = data.to_vec().try_into()?;
        socket.send(Message::Text(data)).await?;
        Ok(())
    }
    async fn send_packet(&self, packet: Self::OutgoingPacket) -> Result<()> {
        self.state.lock().await.send_packet(packet).await
    }
    async fn broadcast(&self, packet: Self::OutgoingPacket) -> Result<()> {
        let mut state = self.state.lock().await;
        state.broadcast(packet).await
    }
}
impl SessionClient {
    pub fn new(
        app_state: Arc<AppState>,
        uuid: PlayerUuid,
        websocket: WebSocket,
    ) -> (Self, impl Future<Output = ()>) {
        let (outgoing_sender, outgoing_receiver) = mpsc::channel(16);
        let state = Arc::new(Mutex::new(SessionState::new(
            app_state,
            uuid,
            outgoing_sender,
        )));
        let fut = Self::run(websocket, state.clone(), outgoing_receiver);
        (Self { state }, fut)
    }
}
