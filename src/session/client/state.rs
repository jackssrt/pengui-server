use std::sync::{Arc, nonpoison::RwLock};

use anyhow::{Result, anyhow};
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::{
    client::{Client, state::ClientState},
    player::{
        Player, badge::BadgeName, badge_slots::BadgeSlots, ids::PlayerUuid, medal::Medals,
        name::PlayerName, rank::Rank, screenshot_limit::ScreenshotLimit,
        traits::FetchForPlayerUuid,
    },
    room,
    server::state::AppState,
    session::client::packet::{IncomingPacket, OutgoingPacket},
};

#[derive(Clone)]
pub struct SessionState {
    state: Arc<AppState>,
    uuid: PlayerUuid,
    outgoing_sender: Sender<OutgoingPacket>,
}

impl SessionState {
    pub const fn new(
        state: Arc<AppState>,
        uuid: PlayerUuid,
        outgoing_sender: Sender<OutgoingPacket>,
    ) -> Self {
        Self {
            state,
            uuid,
            outgoing_sender,
        }
    }
}

impl ClientState for SessionState {
    type IncomingPacket = IncomingPacket;
    type OutgoingPacket = OutgoingPacket;
    async fn process_packet(&mut self, packet: Self::IncomingPacket) -> Result<()> {
        match packet {
            IncomingPacket::SetName(name) => self.handle_name(name).await,
            IncomingPacket::SetPrivateMode(mode) => self.handle_set_private_mode(mode).await,
            IncomingPacket::ClaimExpeditionLocation { name, is_free } => todo!(),
            IncomingPacket::Info() => self.handle_info().await,
            x => {
                tracing::debug!("unimplemented session packet: {:?}", x);
                Ok(())
            }
        }?;
        Ok(())
    }
    async fn broadcast(&mut self, packet: Self::OutgoingPacket) -> Result<()> {
        todo!()
    }
    async fn send_packet(&mut self, packet: Self::OutgoingPacket) -> Result<()> {
        Ok(self.outgoing_sender.send(packet).await?)
    }
}
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct PlayerInfo {
    uuid: PlayerUuid,
    name: Option<PlayerName>,
    rank: Rank,
    badge: Option<BadgeName>,
    #[serde(flatten)]
    badge_slots: BadgeSlots,
    screenshot_limit: ScreenshotLimit,
    medals: Medals,
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
            client.broadcast(packet).await?;
        }
        Ok(())
    }
    async fn handle_set_private_mode(&self, mode: u8) -> Result<()> {
        let player = self.get_player().await?;
        let mut player = player.write();
        player.privacy_settings.singleplayer = mode == 2;
        player.privacy_settings.private = player.privacy_settings.singleplayer || mode == 1;
        Ok(())
    }

    async fn handle_info(&mut self) -> Result<()> {
        let badge_slots = BadgeSlots::fetch_for_player_uuid(&self.state, &self.uuid).await?;
        let screenshot_limit =
            ScreenshotLimit::fetch_for_player_uuid(&self.state, &self.uuid).await?;
        let player = self.get_player().await?;
        self.send_packet({
            let player = player.read();
            let info = PlayerInfo {
                name: player.name.clone(),
                badge: player.badge.clone(),
                badge_slots,
                medals: player.medals.clone(),
                rank: player.rank.clone(),
                screenshot_limit,
                uuid: self.uuid.clone(),
            };
            let output = serde_json::to_string(&info)?;
            OutgoingPacket::Info(output)
        })
        .await?;
        Ok(())
    }
}
