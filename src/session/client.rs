use std::{io::Read, sync::Arc};

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};

use crate::server::state::AppState;

pub struct SessionClient {}
impl SessionClient {
    pub async fn new(state: Arc<AppState>, socket: WebSocket) -> Result<Self> {
        let (mut sink, mut stream) = socket.split();
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(text))) = stream.next().await {
                tracing::trace!("TODO")
            }
        });
        let state = state.clone();
        tokio::spawn(async move {
            sink.send(Message::Text(
                format!("pc\u{ffff}{}", state.players.players.lock().len()).into(),
            ))
            .await
            .unwrap();
        });
        Ok(Self {})
    }
}
