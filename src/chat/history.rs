use anyhow::Result;
use serde::Serialize;

use crate::{
    player::rank::Rank,
    server::{database::Database, state::AppState},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatPlayer {
    pub uuid: String,
    pub name: String,
    pub system_name: String,
    pub rank: Rank,
    pub account: bool,
    pub badge: String,
    pub medals: [isize; 5],
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    #[serde(rename = "msgId")]
    pub message_id: String,
    pub uuid: String,
    #[serde(rename = "prevMapId")]
    pub previous_map_id: String,
    #[serde(rename = "prevLocations")]
    pub previous_locations: String,
    pub x: u16,
    pub y: u16,
    pub contents: String,
    pub timestamp: String,
    #[serde(rename = "party")]
    pub in_party: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatHistory<'a> {
    pub players: &'a [ChatPlayer],
    pub messages: &'a [ChatMessage],
}

pub fn init_history(state: &AppState) {
    if !state.config.is_main_server() {
        return;
    }
    // TODO: schedule deleting old messages
}

pub async fn delete_old_chat_messages(database: &Database) -> Result<()> {
    sqlx::query!(
        "DELETE FROM chatMessages WHERE timestamp < DATE_SUB(UTC_TIMESTAMP(), INTERVAL 1 DAY)"
    )
    .execute(&database.pool)
    .await?;
    Ok(())
}
