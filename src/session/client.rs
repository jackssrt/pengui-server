use crate::connection::Connection;
use anyhow::Result;
use axum::extract::ws::WebSocket;
use std::{boxed::Box, sync::Arc};

pub struct SessionClient {
    pub connection: Connection,
}
impl SessionClient {
    pub fn new(socket: WebSocket) -> Self {
        Self {
            connection: Connection::new(socket, |parts| Box::pin(Self::handle(parts))),
        }
    }
    pub async fn handle(parts: Box<[String]>) -> Result<()> {
        Ok(())
    }
}
