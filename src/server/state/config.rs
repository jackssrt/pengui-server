use std::{collections::HashSet, path::Path, time::Duration};

use anyhow::Result;
use serde::Deserialize;

use crate::room::ids::MapId;

const MAIN_GAME_ID: &str = "2kki";

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Config {
    // game
    pub game_name: Box<str>,
    pub game_path: Box<Path>,

    // database
    pub db_user: Box<str>,
    pub db_pass: Box<str>,
    pub db_addr: Box<str>,
    pub db_name: Box<str>,

    pub sp_rooms: Box<[MapId]>,
    pub bad_sounds: HashSet<Box<str>>,
    pub pictures: HashSet<Box<str>>,
    pub picture_prefixes: Box<[Box<str>]>,
    #[serde(rename = "battle_anim_ids")]
    pub battle_animation_ids: HashSet<u64>,

    // webhooks
    pub chat_webhook: Box<str>,
    pub screenshot_webhook: Box<str>,

    // moderation
    pub moderation: Option<Moderation>,

    // ipc
    #[serde(default)]
    pub ipc: Ipc,

    // vapid keys
    pub vapid_keys: VapidKeys,

    // flags
    pub flags: Flags,
}
impl Config {
    pub fn is_2kki(&self) -> bool {
        *self.game_name == *"2kki"
    }
    pub fn is_main_server(&self) -> bool {
        self.is_2kki()
    }
    pub fn parse(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())?;
        let config = yaml_serde::from_reader(&file)?;

        Ok(config)
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Moderation {
    pub bot_token: Box<str>,
    pub guild_id: Box<str>,
    pub channel_id: Box<str>,
    pub mod_role_id: Box<str>,
}
#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Ipc {
    pub deadline: Duration,
}
impl Default for Ipc {
    fn default() -> Self {
        Self {
            deadline: Duration::from_millis(100),
        }
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VapidKeys {
    pub private: Box<str>,
    pub public: Box<str>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    pub unconsious: bool,
}
