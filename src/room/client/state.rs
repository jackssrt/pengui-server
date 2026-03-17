use std::{
    collections::BTreeMap,
    num::NonZeroU16,
    sync::{Arc, nonpoison::RwLock},
};

use anyhow::{Result, anyhow, bail};
use strum::EnumIs;
use tokio::sync::{Mutex, mpsc::Sender};

use crate::{
    player::Player,
    room::{
        Room,
        client::{
            direction::Direction,
            flash::Flash,
            packet::{IncomingPacket, OutgoingPacket},
        },
        ids::{SwitchId, VariableId},
    },
    server::{rooms::Rooms, state::AppState},
};

use super::packet::{AddPictureData, PictureData};

#[derive(EnumIs)]
enum MovementType {
    Move,
    Jump,
    Teleport,
}
#[derive(EnumIs)]
enum ExtraPictureData {
    Move { duration: u64 },
    Add(AddPictureData),
}

struct SavedPicture(PictureData, AddPictureData);

pub struct ClientState {
    state: Arc<AppState>,
    outgoing_sender: Sender<OutgoingPacket>,
    pub room: Arc<RwLock<Room>>,
    pub player: Arc<RwLock<Player>>,
    pub x: u16,
    pub y: u16,
    pub facing: Direction,
    pub speed: u8,
    pub sync_coords: bool,
    pub flash: Option<Flash>,
    pub transparency: u8,
    pub is_hidden: bool,
    switch_cache: BTreeMap<SwitchId, bool>,
    variable_cache: BTreeMap<VariableId, u16>,
    saved_picture: Option<SavedPicture>,
}

impl ClientState {
    const MAX_PICTURE_ID: u16 = 1000;

    pub fn new(
        state: Arc<AppState>,
        player: Arc<RwLock<Player>>,
        outgoing_sender: Sender<OutgoingPacket>,
        room: Arc<RwLock<Room>>,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            state,
            outgoing_sender,
            room,
            player,
            x: 0,
            y: 0,
            facing: Direction::default(),
            speed: 0,
            sync_coords: false,
            flash: None,
            transparency: 0,
            is_hidden: false,
            switch_cache: BTreeMap::new(),
            variable_cache: BTreeMap::new(),
            saved_picture: None,
        }))
    }

    pub async fn handle_incoming_packet(&mut self, packet: IncomingPacket) -> Result<()> {
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
            IncomingPacket::AnimationCommand => todo!(),
        }?;
        Ok(())
    }

    async fn handle_switch_room(&self, new_room: Option<NonZeroU16>) -> Result<()> {
        // get current room
        // remove self from said room
        if let Some(new_room) = new_room {
            if let Some(room_id) = self.state.assets.is_valid_map_id(new_room) {
                // add client to room
                let rooms = &mut *self.state.rooms.lock();
                let room = Rooms::get_by_id(rooms, room_id);
            }
        } else {
            // ok new room is no room bye bye
        }
        Ok(())
    }

    async fn handle_movement(&mut self, movement_type: MovementType, x: u16, y: u16) -> Result<()> {
        if movement_type.is_move() {
            self.facing = if y < self.y {
                Direction::Up
            } else if x > self.x {
                Direction::Right
            } else if y > self.y {
                Direction::Down
            } else {
                Direction::Left
            }
        }
        (self.x, self.y) = (x, y);
        if movement_type.is_teleport() {
            // TODO checkroomcondition
        }
        if self.sync_coords {
            // TODO check coords condition
        }
        let id = self.player.read().id;
        if movement_type.is_jump() {
            self.outgoing_sender
                .send(OutgoingPacket::Jump {
                    player_id: id,
                    x,
                    y,
                })
                .await?;
        } else {
            self.outgoing_sender
                .send(OutgoingPacket::Move {
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
        let id = self.player.read().id;
        self.outgoing_sender
            .send(OutgoingPacket::ChangeFacingDirection {
                player_id: id,
                direction,
            })
            .await?;
        Ok(())
    }
    async fn handle_speed(&mut self, speed: u8) -> Result<()> {
        let speed = speed.min(10);
        self.speed = speed;
        let id = self.player.read().id;
        self.broadcast(OutgoingPacket::ChangeSpeed {
            player_id: id,
            speed,
        })
        .await?;
        Ok(())
    }

    async fn handle_sprite(&self, sprite: String, sprite_index: u32) -> Result<()> {
        if !self.state.assets.is_valid_sprite(&sprite) {
            return Err(anyhow!("invalid sprite"));
        }
        // this is where the 2kki check would be if it actually did something other than always return true
        let id = {
            let mut player = self.player.write();
            player.game_data.sprite.clone_from(&sprite);
            player.game_data.sprite_index = sprite_index;
            player.id
        };
        self.broadcast(OutgoingPacket::ChangeSprite {
            player_id: id,
            name: sprite,
            index: sprite_index,
        })
        .await?;
        Ok(())
    }

    async fn handle_flash(&self, flash: Flash) -> Result<()> {
        let id = self.player.read().id;
        self.broadcast(OutgoingPacket::PlayerFlash {
            player_id: id,
            flash,
        })
        .await?;
        Ok(())
    }

    async fn handle_repeating_flash(&mut self, flash: Flash) -> Result<()> {
        let id = self.player.read().id;
        self.flash = Some(flash.clone());
        self.broadcast(OutgoingPacket::RepeatingPlayerFlash {
            player_id: id,
            flash,
        })
        .await?;
        Ok(())
    }

    async fn handle_remove_repeating_flash(&mut self) -> Result<()> {
        let id = self.player.read().id;
        self.flash = None;
        self.broadcast(OutgoingPacket::RemoveRepeatingPlayerFlash(id))
            .await?;
        Ok(())
    }

    async fn handle_transparency(&self, transparency: u8) -> Result<()> {
        let id = self.player.read().id;
        // 0 - 7
        let transparency = transparency.min(7);
        self.broadcast(OutgoingPacket::ChangeTransparency(id, transparency))
            .await?;
        Ok(())
    }

    async fn handle_visibility(&mut self, is_hidden: bool) -> Result<()> {
        let id = self.player.read().id;
        self.is_hidden = is_hidden;
        self.broadcast(OutgoingPacket::ChangeSpriteVisibility {
            player_id: id,
            is_hidden,
        })
        .await?;
        Ok(())
    }

    async fn handle_system(&self, system: String) -> Result<()> {
        if !self.state.assets.is_valid_system(&system, false) {
            bail!("invalid system")
        }
        let id = {
            let mut player = self.player.write();
            player.game_data.system.clone_from(&system);
            player.id
        };
        self.broadcast(OutgoingPacket::ChangeSystemGraphic(id, system))
            .await?;
        Ok(())
    }

    async fn handle_play_sound_effect(
        &self,
        name: String,
        volume: u8,
        tempo: u16,
        balance: u8,
    ) -> Result<()> {
        if !self.state.assets.is_valid_sound(&name) {
            bail!("invalid sound");
        }
        let id = self.player.read().id;
        let volume = volume.min(100);
        let tempo = tempo.clamp(10, 400);
        let balance = balance.min(100);
        self.broadcast(OutgoingPacket::PlaySoundEffect {
            player_id: id,
            name,
            volume,
            tempo,
            balance,
        })
        .await?;
        Ok(())
    }
    async fn handle_battle_animation(&self, id: u64) -> Result<()> {
        if !self.state.config.battle_animation_ids.contains(&id) {
            bail!("invalid battle animation id")
        }
        let player_id = self.player.read().id;
        self.broadcast(OutgoingPacket::BattleAnimation(player_id, id))
            .await?;
        Ok(())
    }

    async fn broadcast(&self, packet: OutgoingPacket) -> Result<()> {
        let player = self.player.read();
        if player.moderation_status.is_banned() {
            return Err(anyhow!("player is banned"));
        }
        let room = self.room.read();
        room.players
            .iter()
            .filter(|other| !player.is_blocked_with(&other.read()))
            .filter(|other| !other.read().is_privated_to(&player))
            .filter(|other| player.is_unnamed_player_hidden_by(&other.read()))
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

        Ok(())
    }

    async fn handle_sync_switch(&mut self, switch_id: u16, value: bool) -> Result<()> {
        let switch_id = SwitchId(switch_id);

        // 2kki debug switch
        let is_2kki_debug_switch =
            self.state.config.is_2kki() && switch_id == SwitchId::DEBUG_MODE_2KKI;
        if is_2kki_debug_switch && self.player.read().rank.is_user() && value {
            bail!("you tried to enable debug mode for everyone, don't do that");
        }

        // add to cache
        self.switch_cache.insert(switch_id, value);

        // time trial 2kki
        let is_2kki_time_trial =
            self.state.config.is_2kki() && switch_id == SwitchId::TIME_TRIAL_2KKI;
        if is_2kki_time_trial {
            if value {
                self.outgoing_sender
                    .send(OutgoingPacket::SyncVariable(
                        VariableId::TIME_TRIAL_ELAPSED_2KKI,
                        0,
                    ))
                    .await?;
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
        let saved_picture = if let ExtraPictureData::Add(ref add_picture_data) = extra_picture_data
        {
            SavedPicture(
                PictureData {
                    red,
                    green,
                    blue,
                    saturation,
                    ..picture_data
                },
                add_picture_data.clone(),
            )
        } else {
            self.saved_picture
                .take()
                .ok_or_else(|| anyhow!("tried to modify non-existant picture"))?
        };

        self.broadcast(match extra_picture_data {
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
            .take_if(|SavedPicture(PictureData { id: x, .. }, ..)| *x == id);
        Ok(())
    }
}
