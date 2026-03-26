use std::sync::{Arc, Weak, nonpoison::RwLock};

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use bstr::BStr;
use tokio::sync::{Mutex, mpsc};

use crate::{
    client::{Client, state::ClientState},
    player::Player,
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
        tracing::trace!("<- {:?}", packet);
        state.lock().await.process_packet(packet).await?;
        Ok(())
    }
    async fn handle_outgoing(
        state: &Mutex<Self::State>,
        socket: &mut WebSocket,
        packet: Self::OutgoingPacket,
    ) -> Result<()> {
        tracing::trace!("-> {:?}", packet);
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
        player: Weak<RwLock<Player>>,
        outgoing_sender: mpsc::Sender<OutgoingPacket>,
    ) -> Self {
        let state = Arc::new(Mutex::new(SessionState::new(
            app_state,
            player,
            outgoing_sender,
        )));

        Self { state }
    }
    pub fn run(
        &self,
        socket: WebSocket,
        outgoing_receiver: mpsc::Receiver<OutgoingPacket>,
    ) -> impl std::future::Future<Output = ()> {
        <Self as Client>::run(socket, self.state.clone(), outgoing_receiver)
    }
}
