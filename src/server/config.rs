use anyhow::Result;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;

use crate::room::ids::MapId;

const MAIN_GAME_ID: &str = "2kki";

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Config {
    // game
    pub game_name: String,
    pub game_path: PathBuf,

    // database
    pub db_user: String,
    pub db_pass: String,
    pub db_addr: String,
    pub db_name: String,

    pub sp_rooms: Vec<MapId>,
    pub bad_sounds: HashSet<String>,
    pub pictures: HashSet<String>,
    pub picture_prefixes: Vec<String>,
    #[serde(rename = "battle_anim_ids")]
    pub battle_animation_ids: HashSet<u64>,

    // webhooks
    pub chat_webhook: String,
    pub screenshot_webhook: String,

    // moderation
    pub moderation: Option<Moderation>,

    // ipc
    #[serde(default)]
    pub ipc: Ipc,

    // logging
    #[serde(default)]
    pub logging: Logging,

    // vapid keys
    pub vapid_keys: VapidKeys,

    // flags
    pub flags: Flags,
}
impl Config {
    pub fn is_main_server(&self) -> bool {
        self.game_name == "2kki"
    }
    pub async fn parse(path: impl AsRef<Path>) -> Result<Self> {
        let data = tokio::fs::read(path.as_ref()).await?;
        let data = yaml_serde::from_slice::<Self>(&data)?;

        Ok(data)
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Moderation {
    pub bot_token: String,
    pub guild_id: String,
    pub channel_id: String,
    pub mod_role_id: String,
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
#[derive(Deserialize, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct Logging {
    #[serde(default = "Logging::max_size_default")]
    pub max_size: isize,
    #[serde(default = "Logging::max_backups_default")]
    pub max_backups: isize,
    #[serde(default = "Logging::max_age_default")]
    pub max_age: Duration,
}
impl Logging {
    const fn max_size_default() -> isize {
        50
    }
    const fn max_backups_default() -> isize {
        6
    }
    const fn max_age_default() -> Duration {
        Duration::from_weeks(4)
    }
}
#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VapidKeys {
    pub private: String,
    pub public: String,
}
#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    pub unconsious: bool,
}
