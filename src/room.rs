use std::sync::{Weak, nonpoison::RwLock};

use crate::{
    player::Player,
    room::{ids::MapId, minigame::Minigame},
};

pub mod badges;
pub mod client;
pub mod ids;
pub mod minigame;
pub struct Room {
    pub id: MapId,
    pub is_singleplayer: bool,
    pub players: Vec<Weak<RwLock<Player>>>,
    // TODO
    // pub conditions: Vec<Condition>,
    pub minigame: Option<Minigame>,
}
impl Room {
    pub const fn new(id: MapId) -> Self {
        Self {
            players: Vec::new(),
            is_singleplayer: false,
            minigame: None,
            id,
        }
    }
}
