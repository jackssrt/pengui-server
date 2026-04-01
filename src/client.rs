use std::sync::Arc;

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use serde::Serialize;
use tokio::{
    select,
    sync::{Mutex, mpsc},
};

use crate::client::state::ClientState;

pub mod packet;
pub mod state;

pub trait Client
where
    Self::OutgoingPacket: Serialize,
    Self: Sized,
    Self::State: ClientState,
{
    type OutgoingPacket;
    type State;
    /// returns a future which will run until the connection with the client breaks
    async fn run(
        mut socket: WebSocket,
        state: Arc<Mutex<Self::State>>,
        mut outgoing_receiver: mpsc::UnboundedReceiver<Self::OutgoingPacket>,
    ) {
        loop {
            // these share state, which means if we were to split them up into two tasks
            // they would just contest the mutex instead of actually doing work any faster
            select! {
                Some(packet) = outgoing_receiver.recv() => {
                    if let Err(e) = Self::handle_outgoing(&state, &mut socket, packet).await {
                        tracing::error!("error handling outgoing client packet, {}", e);
                        break;
                    }
                },
                message = socket.recv() => {
                    match message {
                        Some(Ok(message)) => if let Err(e) = Self::handle_incoming(&state, message).await {
                            tracing::error!("error handling incoming client packet, {}", e);
                            break;
                        }
                        Some(Err(e)) => {
                            tracing::error!("error receiving client packet, {}", e);
                            break;
                        },
                        None => {
                            tracing::error!("client closed connection");
                            break;
                        }
                    }
                },
                else => {tracing::error!("broken connection"); break}
            }
        }
    }
    async fn handle_incoming(state: &Mutex<Self::State>, message: Message) -> Result<()>;

    async fn handle_outgoing(
        state: &Mutex<Self::State>,
        socket: &mut WebSocket,
        packet: Self::OutgoingPacket,
    ) -> Result<()>;

    /// Forward to [`Self::State`] to send this [`Client`] a packet
    async fn send_packet(&self, packet: Self::OutgoingPacket) -> Result<()>;
    /// Forward to [`Self::State`] to send other [`Client`]s except this one a packet
    async fn broadcast(&self, packet: Self::OutgoingPacket) -> Result<()>;
}
