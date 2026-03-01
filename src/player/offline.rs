use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    player::{game_data::GameData, ids::PlayerUuid, medal::Medals, rank::Rank},
    room::{badges::Badge, ids::MapId},
};

#[derive(Serialize)]
pub struct DisconnectedPlayer {
    pub uuid: PlayerUuid,
    pub name: Option<String>,
    pub rank: Rank,
    pub is_authenticated: bool,
    pub badge: Option<Badge>,
    #[serde(flatten)]
    pub game_data: GameData,
    pub medals: Option<Medals>,
    #[serde(flatten)]
    pub extra: DisconnectedPlayerExtraData,
}

#[derive(Serialize)]
pub struct DisconnectedPlayerExtraData {
    pub map_id: Option<MapId>,
    pub previous_map_id: Option<MapId>,
    pub previous_locations: Option<String>,
    pub x: i16,
    pub y: i16,
    pub online: bool,
    pub last_active: DateTime<Utc>,
}
