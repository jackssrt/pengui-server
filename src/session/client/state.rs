use std::sync::{Arc, nonpoison::RwLock};

use anyhow::{Result, anyhow};

use crate::{
    client::state::ClientState,
    player::{Player, ids::PlayerUuid, name::PlayerName},
    room,
    server::state::AppState,
    session::client::packet::{IncomingPacket, OutgoingPacket},
};

#[derive(Clone)]
pub struct SessionState {
    state: Arc<AppState>,
    uuid: PlayerUuid,
}

impl SessionState {
    pub const fn new(state: Arc<AppState>, uuid: PlayerUuid) -> Self {
        Self { state, uuid }
    }
}

impl ClientState for SessionState {
    type IncomingPacket = IncomingPacket;
    type OutgoingPacket = OutgoingPacket;
    async fn process_packet(&mut self, packet: Self::IncomingPacket) -> Result<()> {
        match packet {
            IncomingPacket::SetName(name) => self.handle_name(name).await,
            IncomingPacket::GetExpeditions => todo!(),
            IncomingPacket::SayMap(_) => todo!(),
            IncomingPacket::SayParty(_) => todo!(),
            IncomingPacket::SayGlobal(_) => todo!(),
            IncomingPacket::SetPrivateMode(_) => todo!(),
            IncomingPacket::ClaimExpeditionLocation { name, is_free } => todo!(),
            IncomingPacket::Info() => todo!(),
        }?;
        Ok(())
    }
    async fn broadcast(&mut self, packet: Self::OutgoingPacket) -> Result<()> {
        todo!()
    }
}

impl SessionState {
    async fn get_player(&self) -> Result<Arc<RwLock<Player>>> {
        self.state
            .players
            .get_by_uuid(&self.uuid)
            .await
            .ok_or_else(|| anyhow!("invalid player"))
    }
    async fn handle_name(&self, name: String) -> Result<()> {
        let player = self.get_player().await?;
        if let Some((client, packet)) = {
            let mut player = player.write();
            let character_limit = if player.is_authenticated { 12 } else { 10 };
            (!name.is_empty()
                && name.len() <= character_limit
                && name.chars().all(|x| x.is_ascii_alphanumeric()))
            .then(|| {
                player.name = Some(PlayerName(name.clone()));
                player.room_client.as_ref().map(|client| {
                    let client = client.clone();
                    let packet = room::client::packet::OutgoingPacket::Name {
                        player_id: player.id,
                        name,
                    };
                    (client, packet)
                })
            })
            .flatten()
        } {
            client.state.lock().await.broadcast(packet).await?;
        }
        Ok(())
    }
}
