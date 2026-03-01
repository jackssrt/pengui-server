use std::sync::Arc;

use axum::extract::ws::WebSocket;

use crate::{connection::Connection, room::Room};

pub struct RoomClient {
    pub room: Arc<Room>,
    connection: Connection,
    key: u32,
    counter: u32,
}
impl RoomClient {
    pub fn new(room: Arc<Room>, socket: WebSocket) -> Self {
        Self {
            room,
            connection: Connection::new(socket),
            key: 0,
            counter: 0,
        }
    }
}
