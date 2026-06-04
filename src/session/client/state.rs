use std::{
    num::NonZeroU16,
    sync::{Arc, Weak, nonpoison::RwLock},
};

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use sqlx::query;
use strum::EnumIs;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    chat::ids::MessageId,
    client::{Client, state::ClientState},
    player::{
        Player, badge::BadgeName, badge_slots::BadgeSlots, ids::PlayerUuid, medal::Medals,
        name::PlayerName, rank::Rank, screenshot_limit::ScreenshotLimit,
        traits::FetchForPlayerUuid,
    },
    room::{self, client::RoomClient},
    server::state::AppState,
    session::client::packet::{IncomingPacket, OutgoingPacket},
    traits::Random,
};
#[derive(Debug, PartialEq, Eq, Clone, EnumIs)]
enum ChatChannel {
    Map,
    Global,
    Party,
}

#[derive(Clone)]
pub struct SessionState {
    state: &'static AppState,
    player: Weak<RwLock<Player>>,
    outgoing_sender: UnboundedSender<OutgoingPacket>,
}

impl SessionState {
    pub const fn new(
        state: &'static AppState,
        player: Weak<RwLock<Player>>,
        outgoing_sender: UnboundedSender<OutgoingPacket>,
    ) -> Self {
        Self {
            state,
            player,
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
            IncomingPacket::SetPrivateMode(mode) => self.handle_set_private_mode(mode),
            IncomingPacket::GetInfo => self.handle_info().await,
            IncomingPacket::SayMap(content) => self.handle_say(ChatChannel::Map, content).await,
            IncomingPacket::SayGlobal(content) => {
                self.handle_say(ChatChannel::Global, content).await
            }
            IncomingPacket::SayParty(content) => self.handle_say(ChatChannel::Party, content).await,
            IncomingPacket::SetPlayerLocation {
                previous_map_id,
                previous_locations,
            } => {
                self.handle_player_location(previous_map_id, previous_locations)
                    .await
            }
            IncomingPacket::SetLocationColor { location_name } => {
                self.handle_location_color(location_name).await
            }
            x @ (IncomingPacket::ClaimExpeditionLocation { .. }
            | IncomingPacket::GetExpeditions) => {
                tracing::debug!("unimplemented session packet: {:?}", x);
                Ok(())
            }
        }?;
        Ok(())
    }
    async fn broadcast(&mut self, packet: Self::OutgoingPacket) -> Result<()> {
        todo!()
    }
    fn send_packet(&mut self, packet: Self::OutgoingPacket) -> Result<()> {
        Ok(self.outgoing_sender.send(packet)?)
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
    fn get_player(&self) -> Result<Arc<RwLock<Player>>> {
        self.player
            .upgrade()
            .ok_or_else(|| anyhow!("invalid player"))
    }
    async fn handle_name(&self, name: Arc<str>) -> Result<()> {
        let player = self.get_player()?;
        if let Some((client, packet)) = player.with_mut(|player| {
            let character_limit = if player.is_authenticated { 12 } else { 10 };
            (!name.is_empty()
                && name.len() <= character_limit
                && name.chars().all(|x| x.is_ascii_alphanumeric()))
            .then(|| {
                player.name = Some(PlayerName(name));
                player.room_client.as_ref().map(|client| {
                    let packet = room::client::packet::OutgoingPacket::Name {
                        player_id: player.id,
                        #[allow(clippy::unwrap_used)]
                        name: player.name.as_ref().unwrap().clone(),
                    };
                    (client.clone(), packet)
                })
            })
            .flatten()
        }) {
            client.broadcast(packet).await?;
        }
        Ok(())
    }
    fn handle_set_private_mode(&self, mode: u8) -> Result<()> {
        self.get_player()?.with_mut(|player| {
            player.privacy_settings.singleplayer = mode == 2;
            player.privacy_settings.private = player.privacy_settings.singleplayer || mode == 1;
        });
        Ok(())
    }

    async fn handle_info(&mut self) -> Result<()> {
        let player = self.get_player()?;
        let uuid = player.read().uuid.clone();
        let badge_slots = BadgeSlots::fetch_for_player_uuid(self.state, &uuid).await?;
        let screenshot_limit = ScreenshotLimit::fetch_for_player_uuid(self.state, &uuid).await?;
        self.send_packet({
            let info = player.with(|player| PlayerInfo {
                name: player.name.clone(),
                badge: player.badge.clone(),
                badge_slots,
                medals: player.medals.clone(),
                rank: player.rank.clone(),
                screenshot_limit,
                uuid,
            });
            let output = serde_json::to_string(&info)?;
            OutgoingPacket::Info(output)
        })?;
        Ok(())
    }

    async fn handle_say(&mut self, channel: ChatChannel, content: Arc<str>) -> Result<()> {
        let content: Arc<str> = content.trim().into();
        // moved up here because this is most likely to trigger, and cheapest to check
        // the original server also only checks byte length
        // meaning you can only fit 4 men kissing emojis with skin tone modifiers in one message </3
        if content.is_empty() || content.len() > 150 {
            bail!("invalid message");
        }
        // TODO chat filtering
        let player = self.get_player()?;
        let (name, system) = player.with(|player| {
            if channel.is_map() && player.room_client.is_none() {
                bail!("room client does not exist, are you connected to the room ws?")
            }

            if player.moderation_status.is_muted() {
                bail!("player is muted");
            }

            let (Some(name), Some(system)) = (player.name.clone(), player.game_data.system.clone())
            else {
                bail!("no name or system graphic set");
            };
            Ok((name, system))
        })?;
        match channel {
            ChatChannel::Map => {
                self.do_map_say(content, &player).await?;
            }
            ChatChannel::Global => {
                self.do_global_say(content, &player, name, system).await?;
            }
            ChatChannel::Party => {
                if player.read().party_id.is_none() {
                    bail!("player is not in a party");
                }
                let room_client = player
                    .with(|player| {
                        (!player.privacy_settings.should_hide_location).then(|| {
                            player
                                .room_client
                                .as_ref()
                                .map(|client| client.state.clone())
                        })
                    })
                    .flatten();
            }
        }

        Ok(())
    }

    async fn do_global_say(
        &mut self,
        content: Arc<str>,
        player: &RwLock<Player>,
        name: PlayerName,
        system: Arc<str>,
    ) -> Result<()> {
        let room_client = player
            .with(|player| {
                (!player.privacy_settings.should_hide_location).then(|| {
                    player
                        .room_client
                        .as_ref()
                        .map(|client| client.state.clone())
                })
            })
            .flatten();
        let (map_id, previous_map_id, previous_locations, x, y) =
            if let Some(room_client) = room_client {
                let room_client = room_client.lock().await;
                let room = room_client.room.read();
                (
                    room.id.0.get(),
                    room_client.previous_map_id.map_or(0, |id| id.0.get()),
                    room_client.previous_locations.clone(),
                    room_client.position.map(|p| p.x.cast_signed()),
                    room_client.position.map(|p| p.y.cast_signed()),
                )
            } else {
                (0, 0, Arc::default(), None, None)
            };
        let message_id = MessageId::random();
        let is_banned = player.with(|player| player.moderation_status.is_banned());
        if is_banned {
            self.send_packet(player.with(|player| OutgoingPacket::SayGlobal {
                uuid: player.uuid.clone(),
                content: content.clone(),
                map_id,
                previous_map_id,
                previous_locations: previous_locations.clone(),
                x: x.unwrap_or(-1),
                y: y.unwrap_or(-1),
                message_id: message_id.clone(),
            }))?;
        } else {
            self.state
                .players
                .broadcast_session_packet(player.with(|player| OutgoingPacket::PlayerInfo {
                    uuid: player.uuid.clone(),
                    name,
                    system,
                    rank: player.rank.clone(),
                    badge: player.badge.clone(),
                    medals: player.medals.clone(),
                    is_authenticated: player.is_authenticated,
                }))
                .await;
            self.state
                .players
                .broadcast_session_packet(player.with(|player| OutgoingPacket::SayGlobal {
                    uuid: player.uuid.clone(),
                    content: content.clone(),
                    map_id,
                    previous_map_id,
                    previous_locations: previous_locations.clone(),
                    x: x.unwrap_or(-1),
                    y: y.unwrap_or(-1),
                    message_id: message_id.clone(),
                }))
                .await;
        }
        let uuid = player.with(|player| player.uuid.clone());
        query!(
            "INSERT INTO chatMessages (msgId, game, uuid, mapId, prevMapId, prevLocations, x, y, contents) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            message_id.0,
            self.state.config.game_name,
            uuid.0,
            map_id,
            previous_map_id,
            previous_locations,
            x.unwrap_or(-1),
            y.unwrap_or(-1),
            content,
        ).execute(&self.state.database.pool).await?;
        // TODO webhook
        Ok(())
    }

    async fn do_map_say(&mut self, content: Arc<str>, player: &RwLock<Player>) -> Result<()> {
        if !player.read().moderation_status.is_banned() {
            // checked before that room client exists, so we can unwrap here
            #[allow(clippy::unwrap_used)]
            player
                .with(|player| player.room_client.as_ref().unwrap().state.clone())
                .lock()
                .await
                .room
                .clone()
                .with(|room| {
                    room.players
                        .iter()
                        .filter_map(Weak::upgrade)
                        .filter(|other| other.read().uuid != player.read().uuid)
                        .filter(|other| {
                            let player = player.read();
                            let other = other.read();

                            !player.is_blocked_with(&other) && !other.is_privated_to(&player)
                        })
                        .map(|other| {
                            let packet = OutgoingPacket::SayMap {
                                uuid: player.read().uuid.clone(),
                                content: content.clone(),
                            };
                            (other, packet)
                        })
                        .for_each(|(other, packet)| {
                            tokio::spawn(async move {
                                other
                                    .with(|other| other.session_client.state.clone())
                                    .lock()
                                    .await
                                    .send_packet(packet)
                            });
                        });
                });
        }
        self.send_packet(player.with(|player| OutgoingPacket::SayMap {
            uuid: player.uuid.clone(),
            content,
        }))?;
        Ok(())
    }

    async fn handle_player_location(
        &self,
        previous_map_id: u16,
        previous_locations: Arc<str>,
    ) -> Result<()> {
        let player = self.get_player()?;
        let room_client = player
            .with(|player| player.room_client.clone())
            .context("room client does not exist")?;
        let mut state = room_client.state.lock().await;
        // this might cause problems in the future idk
        // the original server doesn't do this
        state.previous_map_id = self.state.assets.is_valid_map_id(
            NonZeroU16::try_from(previous_map_id).context("invalid previous map id")?,
        );
        state.previous_locations = previous_locations;

        // TODO conditions

        Ok(())
    }

    #[allow(clippy::unused_async_trait_impl)]
    async fn handle_location_color(&self, location_name: Arc<str>) -> Result<()> {
        let _ = self.get_room_client()?;
        // TODO events
        // self.send_packet(if let Some(location_color) = event.game_location_colors.get(&location_name) {
        //     OutgoingPacket::LocationColor { key: location_color.key.clone(), value: location_color.value.clone() }
        // } else {
        //     OutgoingPacket::LocationColor { key: Arc::default(), value: Arc::default() }
        // }).await?;

        Ok(())
    }

    fn get_room_client(&self) -> Result<Arc<RoomClient>> {
        let player = self.get_player()?;
        let room_client = player
            .with(|player| player.room_client.clone())
            .context("room client does not exist")?;
        Ok(room_client)
    }
}
