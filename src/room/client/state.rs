use std::{
    collections::BTreeMap,
    num::NonZeroU16,
    sync::{Arc, Weak, nonpoison::RwLock},
};

use anyhow::{Context, Result, anyhow, bail};
use strum::EnumIs;
use tokio::sync::{Mutex, mpsc::UnboundedSender};

use super::packet::{AddPictureData, AnimationCommand, PictureData};
use crate::{
    client::{Client, state::ClientState},
    player::Player,
    room::{
        Room,
        client::{
            cryptography::Cryptography,
            direction::Direction,
            flash::Flash,
            packet::{IncomingPacket, OutgoingPacket},
        },
        ids::{MapId, SwitchId, VariableId},
    },
    server::state::{AppState, rooms::Rooms},
};

#[derive(Debug, PartialEq, Eq, Clone, EnumIs)]
enum MovementType {
    Move,
    Teleport,
    Jump,
}

#[derive(EnumIs)]
enum ExtraPictureData {
    Move { duration: u64 },
    Add(AddPictureData),
}

pub struct SavedPicture(pub PictureData, pub AddPictureData);
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub struct RoomClientState {
    state: &'static AppState,
    outgoing_sender: UnboundedSender<OutgoingPacket>,
    pub room: Arc<RwLock<Room>>,
    pub player: Weak<RwLock<Player>>,
    pub facing: Direction,
    pub speed: Option<u8>,
    pub position: Option<Position>,
    pub sync_coords: bool,
    pub flash: Option<Arc<Flash>>,
    pub transparency: u8,
    pub is_hidden: bool,
    switch_cache: BTreeMap<SwitchId, bool>,
    variable_cache: BTreeMap<VariableId, u16>,
    pub saved_picture: Option<Arc<SavedPicture>>,
    pub cryptography: Cryptography,
    pub previous_map_id: Option<MapId>,
    pub previous_locations: Arc<str>,
}

impl ClientState for RoomClientState {
    type IncomingPacket = IncomingPacket;
    type OutgoingPacket = OutgoingPacket;
    async fn process_packet(&mut self, packet: Self::IncomingPacket) -> Result<()> {
        match packet {
            IncomingPacket::SwitchRoom(new_id) => {
                self.handle_switch_room(NonZeroU16::new(new_id)).await
            }
            IncomingPacket::Move { x, y } => self.handle_movement(MovementType::Move, x, y).await,
            IncomingPacket::Teleport { x, y } => {
                self.handle_movement(MovementType::Teleport, x, y).await
            }
            IncomingPacket::Jump { x, y } => self.handle_movement(MovementType::Jump, x, y).await,
            IncomingPacket::ChangeFacingDirection(direction) => self.handle_facing(direction).await,
            IncomingPacket::PlayerFlash(flash) => self.handle_flash(flash).await,
            IncomingPacket::RepeatingPlayerFlash(flash) => self.handle_repeating_flash(flash).await,
            IncomingPacket::ChangeTransparency(transparency) => {
                self.handle_transparency(transparency).await
            }
            IncomingPacket::ChangeSpriteVisibility { is_hidden } => {
                self.handle_visibility(is_hidden).await
            }
            IncomingPacket::ChangeSystemGraphic(system) => self.handle_system(system).await,
            IncomingPacket::PlaySoundEffect {
                name,
                volume,
                tempo,
                balance,
            } => {
                self.handle_play_sound_effect(name, volume, tempo, balance)
                    .await
            }
            IncomingPacket::BattleAnimation(id) => self.handle_battle_animation(id).await,
            IncomingPacket::ChangeSpeed(speed) => self.handle_speed(speed).await,
            IncomingPacket::ChangeSprite { name, index } => self.handle_sprite(name, index).await,
            IncomingPacket::RemoveRepeatingPlayerFlash => {
                self.handle_remove_repeating_flash().await
            }
            IncomingPacket::AddPicture {
                picture_data,
                add_picture_data,
            } => {
                self.handle_picture(picture_data, ExtraPictureData::Add(add_picture_data))
                    .await
            }
            IncomingPacket::MovePicture {
                picture_data,
                duration,
            } => {
                self.handle_picture(picture_data, ExtraPictureData::Move { duration })
                    .await
            }
            IncomingPacket::RemovePicture(id) => self.handle_remove_picture(id).await,
            IncomingPacket::SyncSwitch { switch_id, value } => {
                self.handle_sync_switch(switch_id, value).await
            }
            IncomingPacket::SyncVariable { variable_id, value } => {
                self.handle_sync_variable(variable_id, value)
            }
            IncomingPacket::SyncEvent {
                is_action,
                event_id,
            } => todo!(),
            IncomingPacket::AnimationCommand(command) => {
                self.handle_animation_command(command).await
            }
        }?;
        Ok(())
    }
    async fn broadcast(&mut self, packet: OutgoingPacket) -> Result<()> {
        let player = self.get_player()?;
        let player = player.read();
        if player.moderation_status.is_banned() {
            return Err(anyhow!("player is banned"));
        }
        self.room.with(|room| {
            room.players
                .iter()
                .filter_map(Weak::upgrade)
                .filter(|other| player.uuid != other.read().uuid)
                .filter(|other| {
                    !player.is_blocked_with(&other.read())
                        && !other.read().is_privated_to(&player)
                        && !player.is_unnamed_player_hidden_by(&other.read())
                })
                .filter_map(|other| {
                    let other = other.read();
                    let room_client = &other.room_client;

                    room_client
                        .as_ref()
                        .map(|room_client| (packet.clone(), room_client.clone()))
                })
                .for_each(|(packet, room_client)| {
                    tokio::spawn(async move {
                        #[allow(clippy::unwrap_used)]
                        room_client.send_packet(packet).await.unwrap();
                    });
                });
        });

        Ok(())
    }
    fn send_packet(&mut self, packet: Self::OutgoingPacket) -> Result<()> {
        Ok(self.outgoing_sender.send(packet)?)
    }
}

impl RoomClientState {
    const MAX_PICTURE_ID: u16 = 1000;

    pub fn new(
        state: &'static AppState,
        room: Arc<RwLock<Room>>,
        player: Weak<RwLock<Player>>,
        outgoing_sender: UnboundedSender<OutgoingPacket>,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            state,
            outgoing_sender,
            room,
            player,
            position: None,
            facing: Direction::default(),
            speed: Some(0),
            sync_coords: false,
            flash: None,
            transparency: 0,
            is_hidden: false,
            switch_cache: BTreeMap::new(),
            variable_cache: BTreeMap::new(),
            saved_picture: None,
            cryptography: Cryptography::new(),
            previous_map_id: None,
            previous_locations: Arc::default(),
        }))
    }

    async fn handle_switch_room(&mut self, new_room: Option<NonZeroU16>) -> Result<()> {
        // remove self from old room
        self.leave_current_room().await?;
        let new_room = new_room.context("invalid room id")?;
        let Some(room_id) = self.state.assets.is_valid_map_id(new_room) else {
            bail!("invalid room id")
        };
        // add client to room
        let rooms = self.state.rooms.rooms.with_mut(|rooms| {
            let room = Rooms::get_by_id(rooms, room_id);
            self.room = room.clone();
        });
        self.join_room().await?;
        Ok(())
    }

    async fn handle_movement(&mut self, movement_type: MovementType, x: i16, y: i16) -> Result<()> {
        let (Ok(x), Ok(y)) = (x.try_into(), y.try_into()) else {
            // position was negative, remove their position
            self.position = None;
            return Ok(());
        };
        if movement_type.is_move()
            && let Some(position) = &self.position
        {
            self.facing = if y < position.y {
                Direction::Up
            } else if x > position.x {
                Direction::Right
            } else if y > position.y {
                Direction::Down
            } else {
                Direction::Left
            }
        }
        self.position = Some(Position { x, y });
        if movement_type.is_teleport() {
            // TODO checkroomcondition
        }
        if self.sync_coords {
            // TODO check coords condition
        }
        let id = self.get_player()?.with(|player| player.id);
        if movement_type.is_jump() {
            self.broadcast(OutgoingPacket::Jump {
                player_id: id,
                x,
                y,
            })
            .await?;
        } else {
            self.broadcast(OutgoingPacket::Move {
                player_id: id,
                x,
                y,
            })
            .await?;
        }
        Ok(())
    }

    async fn handle_facing(&mut self, direction: Direction) -> Result<()> {
        self.facing = direction;
        let id = self.get_player()?.with(|player| player.id);
        self.broadcast(OutgoingPacket::ChangeFacingDirection {
            player_id: id,
            direction,
        })
        .await?;
        Ok(())
    }
    async fn handle_speed(&mut self, speed: u8) -> Result<()> {
        let speed = speed.min(10);
        self.speed = Some(speed);
        let id = self.get_player()?.with(|player| player.id);
        self.broadcast(OutgoingPacket::ChangeSpeed {
            player_id: id,
            speed,
        })
        .await?;
        Ok(())
    }

    async fn handle_sprite(&mut self, sprite: Arc<str>, sprite_index: u32) -> Result<()> {
        if !self.state.assets.is_valid_sprite(&sprite) {
            return Err(anyhow!("invalid sprite"));
        }
        // this is where the 2kki check would be if it actually did something other than always return true
        let id = self.get_player()?.with_mut(|player| {
            player.game_data.sprite.clone_from(&sprite);
            player.game_data.sprite_index = Some(sprite_index);
            player.id
        });
        self.broadcast(OutgoingPacket::ChangeSprite {
            player_id: id,
            name: sprite,
            index: sprite_index,
        })
        .await?;
        Ok(())
    }

    async fn handle_flash(&mut self, flash: Flash) -> Result<()> {
        self.broadcast(
            self.get_player()?
                .with(|player| OutgoingPacket::PlayerFlash {
                    player_id: player.id,
                    flash,
                }),
        )
        .await?;
        Ok(())
    }

    async fn handle_repeating_flash(&mut self, flash: Flash) -> Result<()> {
        let flash = Arc::new(flash);
        self.flash = Some(flash.clone());
        self.broadcast(
            self.get_player()?
                .with(|player| OutgoingPacket::RepeatingPlayerFlash {
                    player_id: player.id,
                    flash,
                }),
        )
        .await?;
        Ok(())
    }

    async fn handle_remove_repeating_flash(&mut self) -> Result<()> {
        self.flash = None;
        self.broadcast(OutgoingPacket::RemoveRepeatingPlayerFlash(
            self.get_player()?.with(|player| player.id),
        ))
        .await?;
        Ok(())
    }

    async fn handle_transparency(&mut self, transparency: u8) -> Result<()> {
        // 0 - 7
        let transparency = transparency.min(7);
        self.broadcast(OutgoingPacket::ChangeTransparency(
            self.get_player()?.with(|player| player.id),
            transparency,
        ))
        .await?;
        Ok(())
    }

    async fn handle_visibility(&mut self, is_hidden: bool) -> Result<()> {
        self.is_hidden = is_hidden;
        self.broadcast(OutgoingPacket::ChangeSpriteVisibility(
            self.get_player()?.with(|player| player.id),
            is_hidden,
        ))
        .await?;
        Ok(())
    }

    async fn handle_system(&mut self, system: Arc<str>) -> Result<()> {
        if !self.state.assets.is_valid_system(&system) {
            bail!("invalid system")
        }
        let id = self.get_player()?.with_mut(|player| {
            player.game_data.system = Some(system.clone());
            player.id
        });
        self.broadcast(OutgoingPacket::ChangeSystemGraphic(id, system))
            .await?;
        Ok(())
    }

    async fn handle_play_sound_effect(
        &mut self,
        name: Arc<str>,
        volume: u8,
        tempo: u16,
        balance: u8,
    ) -> Result<()> {
        if !self.state.assets.is_valid_sound(&name) {
            bail!("invalid sound");
        }
        let volume = volume.min(100);
        let tempo = tempo.clamp(10, 400);
        let balance = balance.min(100);
        self.broadcast(OutgoingPacket::PlaySoundEffect {
            player_id: self.get_player()?.with(|player| player.id),
            name,
            volume,
            tempo,
            balance,
        })
        .await?;
        Ok(())
    }
    async fn handle_battle_animation(&mut self, id: u64) -> Result<()> {
        if !self.state.config.battle_animation_ids.contains(&id) {
            bail!("invalid battle animation id")
        }
        self.broadcast(OutgoingPacket::BattleAnimation(
            self.get_player()?.with(|player| player.id),
            id,
        ))
        .await?;
        Ok(())
    }

    async fn handle_sync_switch(&mut self, switch_id: u16, value: bool) -> Result<()> {
        let player = self.get_player()?;
        let switch_id = SwitchId(switch_id);

        // 2kki debug switch
        let is_2kki_debug_switch =
            self.state.config.is_2kki() && switch_id == SwitchId::DEBUG_MODE_2KKI;
        if is_2kki_debug_switch && player.read().rank.is_user() && value {
            bail!("you tried to enable debug mode for everyone, don't do that");
        }

        // add to cache
        self.switch_cache.insert(switch_id, value);

        // time trial 2kki
        let is_2kki_time_trial =
            self.state.config.is_2kki() && switch_id == SwitchId::TIME_TRIAL_2KKI;
        if is_2kki_time_trial {
            if value {
                self.outgoing_sender.send(OutgoingPacket::SyncVariable(
                    VariableId::TIME_TRIAL_ELAPSED_2KKI,
                    0,
                ))?;
            }
            return Ok(());
        }

        // TODO: minigame syncing, gonna rewrite soon:tm:
        // TODO: condition syncing

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn handle_sync_variable(&mut self, variable_id: u16, value: u16) -> Result<()> {
        self.variable_cache.insert(VariableId(variable_id), value);
        // TODO: 2kki time trial
        // TODO: minigames
        // TODO: conditions
        Ok(())
    }

    async fn handle_picture(
        &mut self,
        picture_data: PictureData,
        extra_picture_data: ExtraPictureData,
    ) -> Result<()> {
        let extra_picture_data = Arc::new(extra_picture_data);
        // TODO: tbh this whole picture system feels weird
        // since i don't really know how it's used
        // there's prob a lot of bugs in here
        if extra_picture_data.is_add() {
            // TODO: conditions
            if !self
                .state
                .assets
                .is_valid_picture(&picture_data.picture_name)
            {
                bail!("invalid picture")
            }
        }
        if picture_data.picture_name.is_empty() {
            bail!("invalid picture name")
        }
        if picture_data.id > Self::MAX_PICTURE_ID {
            // TODO: better error handling here
            bail!("too many pictures")
        }
        let red = picture_data.red.min(200);
        let green = picture_data.green.min(200);
        let blue = picture_data.blue.min(200);
        let saturation = picture_data.saturation.min(200);
        let saved_picture = if let ExtraPictureData::Add(ref add_picture_data) = *extra_picture_data
        {
            Arc::new(SavedPicture(
                PictureData {
                    red,
                    green,
                    blue,
                    saturation,
                    ..picture_data
                },
                add_picture_data.clone(),
            ))
        } else {
            self.saved_picture
                .take()
                .ok_or_else(|| anyhow!("tried to modify non-existant picture"))?
        };

        self.broadcast(match *extra_picture_data {
            ExtraPictureData::Add(_) => OutgoingPacket::AddPicture {
                picture_data: saved_picture.0.clone(),
                add_picture_data: saved_picture.1.clone(),
            },
            ExtraPictureData::Move { duration } => OutgoingPacket::MovePicture {
                picture_data: saved_picture.0.clone(),
                duration,
            },
        })
        .await?;

        if !saved_picture.1.spritesheet_play_once {
            self.saved_picture = Some(saved_picture);
        }
        Ok(())
    }

    async fn handle_remove_picture(&mut self, id: u16) -> Result<()> {
        self.saved_picture
            .take_if(|saved_picture| saved_picture.0.id == id);
        Ok(())
    }

    async fn handle_animation_command(&mut self, command: AnimationCommand) -> Result<()> {
        self.broadcast(OutgoingPacket::AnimationCommand(
            self.get_player()?.with(|player| player.id),
            command,
        ))
        .await?;
        Ok(())
    }
    pub async fn leave_current_room(&mut self) -> Result<()> {
        let player = self.get_player()?;
        tracing::trace!(
            "removing player {:?} from room {:?}",
            player.read().id,
            self.room.read().id
        );
        self.room.with_mut(|room| {
            let index = room.players.iter().position(|p| {
                p.upgrade()
                    .is_some_and(|p| p.read().uuid == player.read().uuid)
            });

            let Some(index) = index else {
                tracing::warn!(
                    "tried to remove player {:?} from room {:?} but they weren't in it",
                    player.read().id,
                    room.id
                );
                return;
            };
            room.players.remove(index);
            tracing::info!("room {:?} now has {} players", room.id, room.players.len());
        });
        self.broadcast_disconnect_packet().await?;
        Ok(())
    }

    pub async fn join_room(&mut self) -> Result<()> {
        let player = self.get_player()?;
        tracing::trace!(
            "adding player {:?} to room {:?}",
            player.read().id,
            self.room.read().id
        );
        self.send_room_id_packet()?;
        if !self.room.read().is_singleplayer {
            self.broadcast_connect_packet().await?;
            self.send_other_clients_packets().await?;
            if let (Some(name), id) = player.with(|player| (player.name.clone(), player.id)) {
                self.broadcast(OutgoingPacket::Name {
                    player_id: id,
                    name,
                })
                .await?;
            }
        }
        self.room.write().players.push(Arc::downgrade(&player));
        self.room.with(|room| {
            tracing::info!("room {:?} now has {} players", room.id, room.players.len());
        });
        Ok(())
    }

    async fn broadcast_connect_packet(&mut self) -> Result<()> {
        let packet = self.get_player()?.with(|player| OutgoingPacket::Connect {
            player_id: player.id,
            player_uuid: player.uuid.clone(),
            rank: player.rank.clone(),
            is_authenticated: player.is_authenticated,
            badge: player.badge.clone(),
            medals: player.medals.clone(),
        });
        self.broadcast(packet).await?;
        Ok(())
    }

    async fn send_other_clients_packets(&mut self) -> Result<()> {
        let player = self.get_player()?;
        for (other, other_room_client) in {
            self.room.with(|room| {
                room.players
                    .iter()
                    .filter_map(Weak::upgrade)
                    .filter(|other| {
                        let other = other.read();
                        let player = player.read();
                        !player.is_blocked_with(&other)
                            && !player.is_privated_to(&other)
                            && !player.is_unnamed_player_hidden_by(&other)
                    })
                    .map(|other| {
                        let other_guard = other.read();
                        let room_client = other_guard.room_client.clone();
                        (other.clone(), room_client)
                    })
                    // have to collect here cause otherwise the guard for room is still used
                    .collect::<Vec<_>>()
            })
        } {
            self.send_packet(other.with(|other| OutgoingPacket::Connect {
                player_id: other.id,
                player_uuid: other.uuid.clone(),
                rank: other.rank.clone(),
                is_authenticated: other.is_authenticated,
                badge: other.badge.clone(),
                medals: other.medals.clone(),
            }))?;
            if let Some(other_room_client) = other_room_client {
                let position = { other_room_client.state.lock().await.position };
                if let Some(position) = &position {
                    self.send_packet(other.with(|other| OutgoingPacket::Move {
                        player_id: other.id,
                        x: position.x,
                        y: position.y,
                    }))?;
                }
                let facing = {
                    let other_room_client = other_room_client.state.lock().await;
                    other_room_client.facing
                };
                if facing != Direction::default() {
                    self.send_packet(other.with(|other| OutgoingPacket::ChangeFacingDirection {
                        player_id: other.id,
                        direction: facing,
                    }))?;
                }
                let speed = other_room_client.state.lock().await.speed;
                if let Some(speed) = speed {
                    self.send_packet(other.with(|other| OutgoingPacket::ChangeSpeed {
                        player_id: other.id,
                        speed,
                    }))?;
                }
                // ok to clone the name here since we would need to clone it later anyway to send the packet
                // cloning a None is cheap
                if let Some(name) = { other.read().name.clone() } {
                    self.send_packet(other.with(|other| OutgoingPacket::Name {
                        player_id: other.id,
                        name,
                    }))?;
                }
                if let Some(sprite_index) = { other.read().game_data.sprite_index } {
                    self.send_packet(other.with(|other| OutgoingPacket::ChangeSprite {
                        player_id: other.id,
                        name: other.game_data.sprite.clone(),
                        index: sprite_index,
                    }))?;
                }
                let flash = { other_room_client.state.lock().await.flash.clone() };
                if let Some(flash) = flash {
                    self.send_packet({
                        OutgoingPacket::RepeatingPlayerFlash {
                            player_id: other.read().id,
                            flash: flash.clone(),
                        }
                    })?;
                }
                let transparency = other_room_client.state.lock().await.transparency;
                if transparency != 0 {
                    self.send_packet({
                        OutgoingPacket::ChangeTransparency(other.read().id, transparency)
                    })?;
                }
                let is_hidden = other_room_client.state.lock().await.is_hidden;
                if is_hidden {
                    self.send_packet(
                        other.with(|other| OutgoingPacket::ChangeSpriteVisibility(other.id, true)),
                    )?;
                }
                let system = other.read().game_data.system.clone();
                if let Some(system) = system {
                    self.send_packet(
                        other.with(|other| OutgoingPacket::ChangeSystemGraphic(other.id, system)),
                    )?;
                }
                if let Some(picture) = &other_room_client.state.lock().await.saved_picture {
                    self.send_packet(OutgoingPacket::AddPicture {
                        picture_data: picture.0.clone(),
                        add_picture_data: picture.1.clone(),
                    })?;
                }
            }
        }
        Ok(())
    }

    async fn broadcast_disconnect_packet(&mut self) -> Result<()> {
        self.broadcast(OutgoingPacket::Disconnection {
            player_id: self.get_player()?.with(|player| player.id),
        })
        .await?;
        Ok(())
    }

    fn send_room_id_packet(&mut self) -> Result<()> {
        self.send_packet(OutgoingPacket::RoomId(self.room.with(|room| room.id)))?;
        Ok(())
    }

    fn get_player(&self) -> Result<Arc<RwLock<Player>>> {
        self.player
            .upgrade()
            .ok_or_else(|| anyhow!("player not found"))
    }
}
