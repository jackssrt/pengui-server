use std::{
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
            packet::{IncomingRoomPacket, OutgoingRoomPacket},
        },
    },
    server::{rooms::Rooms, state::AppState},
};

#[derive(EnumIs)]
enum MovementType {
    Move,
    Jump,
    Teleport,
}
pub struct RoomClientState {
    state: Arc<AppState>,
    outgoing_sender: Sender<OutgoingRoomPacket>,
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
}

impl RoomClientState {
    pub fn new(
        state: Arc<AppState>,
        player: Arc<RwLock<Player>>,
        outgoing_sender: Sender<OutgoingRoomPacket>,
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
        }))
    }

    pub async fn handle_incoming_packet(&mut self, packet: IncomingRoomPacket) -> Result<()> {
        match packet {
            IncomingRoomPacket::SwitchRoom(new_id) => {
                self.handle_switch_room(NonZeroU16::new(new_id)).await
            }
            IncomingRoomPacket::Move { x, y } => {
                self.handle_movement(MovementType::Move, x, y).await
            }
            IncomingRoomPacket::Teleport { x, y } => {
                self.handle_movement(MovementType::Teleport, x, y).await
            }
            IncomingRoomPacket::Jump { x, y } => {
                self.handle_movement(MovementType::Jump, x, y).await
            }
            IncomingRoomPacket::ChangeFacingDirection(direction) => {
                self.handle_facing(direction).await
            }
            IncomingRoomPacket::PlayerFlash(flash) => self.handle_flash(flash).await,
            IncomingRoomPacket::RepeatingPlayerFlash(flash) => {
                self.handle_repeating_flash(flash).await
            }
            IncomingRoomPacket::ChangeTransparency(transparency) => {
                self.handle_transparency(transparency).await
            }
            IncomingRoomPacket::ChangeSpriteVisibility { is_hidden } => {
                self.handle_visibility(is_hidden).await
            }
            IncomingRoomPacket::ChangeSystemGraphic(system) => self.handle_system(system).await,
            IncomingRoomPacket::PlaySoundEffect {
                name,
                volume,
                tempo,
                balance,
            } => {
                self.handle_play_sound_effect(name, volume, tempo, balance)
                    .await
            }
            IncomingRoomPacket::BattleAnimation(id) => self.handle_battle_animation(id).await,
            IncomingRoomPacket::ChangeSpeed(speed) => self.handle_speed(speed).await,
            IncomingRoomPacket::ChangeSprite { name, index } => {
                self.handle_sprite(name, index).await
            }
            IncomingRoomPacket::RemoveRepeatingPlayerFlash => {
                self.handle_remove_repeating_flash().await
            }
            IncomingRoomPacket::AddPicture {
                id,
                pos_x,
                pos_y,
                map_x,
                map_y,
                pan_x,
                pan_y,
                magnify,
                top_transparency,
                bottom_transparency,
                red,
                green,
                blue,
                saturation,
                effect_mode,
                effect_power,
                picture_name,
                use_transparent_color,
                fixed_to_map,
                spritesheet_rows,
                spritesheet_cols,
                spritesheet_frame,
                spritesheet_speed,
                spritesheet_play_once,
                map_layer,
                battle_layer,
                flags,
                blend_mode,
                flip_x,
                flip_y,
                origin,
            } => todo!(),
            IncomingRoomPacket::MovePicture {
                id,
                pos_x,
                pos_y,
                map_x,
                map_y,
                pan_x,
                pan_y,
                magnify,
                top_transparency,
                bottom_transparency,
                red,
                green,
                blue,
                saturation,
                effect_mode,
                effect_power,
                picture_name,
            } => todo!(),
            IncomingRoomPacket::RemovePicture(_) => todo!(),
            IncomingRoomPacket::SyncSwitch { switch_id, value } => todo!(),
            IncomingRoomPacket::SyncVariable { variable_id, value } => todo!(),
            IncomingRoomPacket::SyncEvent {
                is_action,
                event_id,
            } => todo!(),
            IncomingRoomPacket::AnimationCommand => todo!(),
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
                .send(OutgoingRoomPacket::Jump {
                    player_id: id,
                    x,
                    y,
                })
                .await?;
        } else {
            self.outgoing_sender
                .send(OutgoingRoomPacket::Move {
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
            .send(OutgoingRoomPacket::ChangeFacingDirection {
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
        self.broadcast(OutgoingRoomPacket::ChangeSpeed {
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
        self.broadcast(OutgoingRoomPacket::ChangeSprite {
            player_id: id,
            name: sprite,
            index: sprite_index,
        })
        .await?;
        Ok(())
    }

    async fn handle_flash(&self, flash: Flash) -> Result<()> {
        let id = self.player.read().id;
        self.broadcast(OutgoingRoomPacket::PlayerFlash {
            player_id: id,
            flash,
        })
        .await?;
        Ok(())
    }

    async fn handle_repeating_flash(&mut self, flash: Flash) -> Result<()> {
        let id = self.player.read().id;
        self.flash = Some(flash.clone());
        self.broadcast(OutgoingRoomPacket::RepeatingPlayerFlash {
            player_id: id,
            flash,
        })
        .await?;
        Ok(())
    }

    async fn handle_remove_repeating_flash(&mut self) -> Result<()> {
        let id = self.player.read().id;
        self.flash = None;
        self.broadcast(OutgoingRoomPacket::RemoveRepeatingPlayerFlash(id))
            .await?;
        Ok(())
    }

    async fn handle_transparency(&self, transparency: u8) -> Result<()> {
        let id = self.player.read().id;
        // 0 - 7
        let transparency = transparency.min(7);
        self.broadcast(OutgoingRoomPacket::ChangeTransparency(id, transparency))
            .await?;
        Ok(())
    }

    async fn handle_visibility(&mut self, is_hidden: bool) -> Result<()> {
        let id = self.player.read().id;
        self.is_hidden = is_hidden;
        self.broadcast(OutgoingRoomPacket::ChangeSpriteVisibility {
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
        self.broadcast(OutgoingRoomPacket::ChangeSystemGraphic(id, system))
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
        self.broadcast(OutgoingRoomPacket::PlaySoundEffect {
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
        self.broadcast(OutgoingRoomPacket::BattleAnimation(player_id, id))
            .await?;
        Ok(())
    }

    async fn broadcast(&self, packet: OutgoingRoomPacket) -> Result<()> {
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
}
