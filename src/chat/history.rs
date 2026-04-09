use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{StreamExt, TryStreamExt};
use serde::Serialize;

use crate::{
    chat::ids::MessageId,
    party::ids::PartyId,
    player::medal::Medals,
    server::state::{AppState, database::Database},
};

pub fn init_history(state: &'static AppState) {
    if !state.config.is_main_server() {
        return;
    }
    // TODO: schedule deleting old messages using something like cron
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatPlayer {
    pub uuid: String,
    pub name: String,
    pub system_name: String,
    pub rank: i32,
    #[serde(rename = "account")]
    pub is_authenticated: bool,
    pub badge: String,
    pub medals: Medals,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "msgId")]
    pub id: String,
    pub uuid: String,
    pub map_id: Option<String>,
    #[serde(rename = "prevMapId")]
    pub previous_map_id: Option<String>,
    #[serde(rename = "prevLocations")]
    pub previous_locations: Option<String>,
    pub x: i32,
    pub y: i32,
    pub contents: String,
    #[serde(skip)]
    pub raw_timestamp: DateTime<Utc>,
    pub timestamp: String,
    #[serde(rename = "party")]
    pub in_party: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatHistory {
    pub players: Vec<ChatPlayer>,
    pub messages: Vec<Message>,
}

struct MessageHistoryRow {
    message_id: String,
    uuid: String,
    map_id: Option<String>,
    previous_map_id: Option<String>,
    previous_locations: Option<String>,
    x: i32,
    y: i32,
    contents: String,
    timestamp: DateTime<Utc>,
    party_value: i32,
}

pub async fn get_chat_history(
    state: &'static AppState,
    party_id: Option<PartyId>,
    global_message_limit: u8,
    party_message_limit: u8,
    last_message_id: Option<MessageId>,
) -> Result<ChatHistory> {
    let messages = get_message_history(
        state,
        party_id.clone(),
        global_message_limit,
        party_message_limit,
        last_message_id,
    )
    .await?;

    let first_message_timestamp = messages.first().map(|m| m.raw_timestamp);
    let last_message_timestamp = messages.last().map(|m| m.raw_timestamp);
    let timestamps = first_message_timestamp.zip(last_message_timestamp);
    let players = if let Some(timestamps) = timestamps {
        get_player_history(state, party_id, timestamps).await?
    } else {
        Vec::new()
    };
    Ok(ChatHistory { players, messages })
}

#[allow(clippy::too_many_lines)]
async fn get_message_history(
    state: &'static AppState,
    party_id: Option<PartyId>,
    global_message_limit: u8,
    party_message_limit: u8,
    last_message_id: Option<MessageId>,
) -> Result<Vec<Message>> {
    // im sorry... but at least it's compiler checked? :D
    let rows_stream = match (party_id, last_message_id) {
        (None, None) => sqlx::query_as!(MessageHistoryRow, "
        (SELECT cm.msgId as message_id, cm.uuid, cm.mapId as map_id, cm.prevMapId as previous_map_id, cm.prevLocations as previous_locations, cm.x, cm.y, cm.contents, cm.timestamp, 0 as party_value
        FROM chatMessages cm
            JOIN players pd ON pd.uuid = cm.uuid
            JOIN playerGameData pgd ON pgd.uuid = pd.uuid AND pgd.game = cm.game
        WHERE
            cm.game = ? 
            AND pd.banned = 0
            AND cm.partyId IS NULL
            AND (
                pgd.lastGlobalMsgId IS NULL
                OR cm.timestamp > (
                    SELECT cmg.timestamp FROM chatMessages cmg WHERE cmg.msgId = pgd.lastGlobalMsgId
                )
            )
        ORDER BY 9 DESC
        LIMIT ?) ORDER BY 9
        ", state.config.game_name, global_message_limit).fetch(&state.database.pool),
        (None, Some(last_message_id)) => sqlx::query_as!(MessageHistoryRow, "
        (SELECT cm.msgId as message_id, cm.uuid, cm.mapId as map_id, cm.prevMapId as previous_map_id, cm.prevLocations as previous_locations, cm.x, cm.y, cm.contents, cm.timestamp, 0 as party_value
        FROM chatMessages cm
            JOIN players pd ON pd.uuid = cm.uuid
            JOIN playerGameData pgd ON pgd.uuid = pd.uuid AND pgd.game = cm.game
        WHERE
            cm.game = ? 
            AND pd.banned = 0
            AND cm.timestamp > (SELECT cm2.timestamp FROM chatMessages cm2 WHERE cm2.msgId = ?)
            AND cm.partyId IS NULL
            AND (
                pgd.lastGlobalMsgId IS NULL
                OR cm.timestamp > (
                    SELECT cmg.timestamp FROM chatMessages cmg WHERE cmg.msgId = pgd.lastGlobalMsgId
                )
            )
        ORDER BY 9 DESC
        LIMIT ?) ORDER BY 9
        ", state.config.game_name, last_message_id.0.as_ref(), global_message_limit).fetch(&state.database.pool),
        (Some(party_id), None) => sqlx::query_as!(MessageHistoryRow, "
        (SELECT cm.msgId as message_id, cm.uuid, cm.mapId as map_id, cm.prevMapId as previous_map_id, cm.prevLocations as previous_locations, cm.x, cm.y, cm.contents, cm.timestamp, 0 as party_value
        FROM chatMessages cm
            JOIN players pd ON pd.uuid = cm.uuid
            JOIN playerGameData pgd ON pgd.uuid = pd.uuid AND pgd.game = cm.game
        WHERE
            cm.game = ? 
            AND pd.banned = 0
            AND cm.partyId IS NULL
            AND (
                pgd.lastGlobalMsgId IS NULL
                OR cm.timestamp > (
                    SELECT cmg.timestamp FROM chatMessages cmg WHERE cmg.msgId = pgd.lastGlobalMsgId
                )
            )
        ORDER BY 9 DESC
        LIMIT ?)
        UNION
        (SELECT cm.msgId as message_id, cm.uuid, cm.mapId as map_id, cm.prevMapId as previous_map_id, cm.prevLocations as previous_locations, cm.x, cm.y, cm.contents, cm.timestamp, 0 as party_value
        FROM chatMessages cm
            JOIN players pd ON pd.uuid = cm.uuid
            JOIN playerGameData pgd ON pgd.uuid = pd.uuid AND pgd.game = cm.game
        WHERE
            cm.game = ? 
            AND pd.banned = 0
            AND cm.partyId = ?
            AND (
                pgd.lastPartyMsgId IS NULL
                OR cm.timestamp > (
                    SELECT cmp.timestamp FROM chatMessages cmp WHERE cmp.msgId = pgd.lastPartyMsgId
                )
            )
        ORDER BY 9 DESC
        LIMIT ?)
        ORDER BY 9
        ", state.config.game_name, global_message_limit, state.config.game_name, party_id.0, party_message_limit).fetch(&state.database.pool),
        (Some(party_id), Some(last_message_id)) => sqlx::query_as!(MessageHistoryRow, "
        (SELECT cm.msgId as message_id, cm.uuid, cm.mapId as map_id, cm.prevMapId as previous_map_id, cm.prevLocations as previous_locations, cm.x, cm.y, cm.contents, cm.timestamp, 0 as party_value
        FROM chatMessages cm
            JOIN players pd ON pd.uuid = cm.uuid
            JOIN playerGameData pgd ON pgd.uuid = pd.uuid AND pgd.game = cm.game
        WHERE
            cm.game = ? 
            AND pd.banned = 0
            AND cm.timestamp > (SELECT cm2.timestamp FROM chatMessages cm2 WHERE cm2.msgId = ?)
            AND cm.partyId IS NULL
            AND (
                pgd.lastGlobalMsgId IS NULL
                OR cm.timestamp > (
                    SELECT cmg.timestamp FROM chatMessages cmg WHERE cmg.msgId = pgd.lastGlobalMsgId
                )
            )
        ORDER BY 9 DESC
        LIMIT ?)
        UNION
        (SELECT cm.msgId as message_id, cm.uuid, cm.mapId as map_id, cm.prevMapId as previous_map_id, cm.prevLocations as previous_locations, cm.x, cm.y, cm.contents, cm.timestamp, 0 as party_value
        FROM chatMessages cm
            JOIN players pd ON pd.uuid = cm.uuid
            JOIN playerGameData pgd ON pgd.uuid = pd.uuid AND pgd.game = cm.game
        WHERE
            cm.game = ? 
            AND pd.banned = 0
            AND cm.timestamp > (SELECT cm2.timestamp FROM chatMessages cm2 WHERE cm2.msgId = ?)
            AND cm.partyId = ?
            AND (
                pgd.lastPartyMsgId IS NULL
                OR cm.timestamp > (
                    SELECT cmp.timestamp FROM chatMessages cmp WHERE cmp.msgId = pgd.lastPartyMsgId
                )
            )
        ORDER BY 9 DESC
        LIMIT ?)
        ORDER BY 9
    ", state.config.game_name, last_message_id.0.as_ref(), global_message_limit, state.config.game_name, last_message_id.0.as_ref(), party_id.0, party_message_limit).fetch(&state.database.pool)
    };
    Ok(rows_stream
        .filter_map(async |x| x.ok())
        .map(|row| Message {
            id: row.message_id,
            uuid: row.uuid,
            map_id: row.map_id,
            previous_map_id: row.previous_map_id,
            previous_locations: row.previous_locations,
            x: row.x,
            y: row.y,
            contents: row.contents,
            // this is kinda stupid but whatever
            raw_timestamp: row.timestamp,
            timestamp: row
                .timestamp
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            in_party: row.party_value != 0,
        })
        .collect()
        .await)
}
struct PlayerRow {
    uuid: String,
    name: String,
    system_name: String,
    rank: i32,
    is_authenticated: i32,
    badge: String,
    medal_count_bronze: Option<i8>,
    medal_count_silver: Option<i8>,
    medal_count_gold: Option<i8>,
    medal_count_platinum: Option<i8>,
    medal_count_diamond: Option<i8>,
}
async fn get_player_history(
    state: &'static AppState,
    party_id: Option<PartyId>,
    (first_timestamp, last_timestamp): (DateTime<Utc>, DateTime<Utc>),
) -> Result<Vec<ChatPlayer>> {
    let rows_stream = party_id.map_or_else(
        || {
            sqlx::query_as!(
                PlayerRow,
                "SELECT DISTINCT
                pd.uuid,
                COALESCE(a.user, pgd.name) as name,
                pd.rank,
                CASE WHEN a.user IS NULL THEN 0 ELSE 1 END as is_authenticated,
                COALESCE(a.badge, '') as badge,
                pgd.systemName as system_name,
                pgd.medalCountBronze as medal_count_bronze,
                pgd.medalCountSilver as medal_count_silver,
                pgd.medalCountGold as medal_count_gold,
                pgd.medalCountPlatinum as medal_count_platinum,
                pgd.medalCountDiamond as medal_count_diamond
            FROM players pd
                JOIN playerGameData pgd ON pgd.uuid = pd.uuid
                LEFT JOIN accounts a ON a.uuid = pd.uuid
            WHERE
                pgd.game = ?
                AND EXISTS (
                    SELECT cm.uuid FROM chatMessages cm
                    WHERE cm.uuid = pd.uuid
                        AND cm.game = pgd.game
                        AND cm.timestamp BETWEEN ? AND ?
                        AND cm.partyId IS NULL
                )",
                state.config.game_name,
                first_timestamp,
                last_timestamp
            )
            .fetch(&state.database.pool)
        },
        |party_id| {
            sqlx::query_as!(
                PlayerRow,
                "SELECT DISTINCT
                pd.uuid,
                COALESCE(a.user, pgd.name) as name,
                pd.rank,
                CASE WHEN a.user IS NULL THEN 0 ELSE 1 END as is_authenticated,
                COALESCE(a.badge, '') as badge,
                pgd.systemName as system_name,
                pgd.medalCountBronze as medal_count_bronze,
                pgd.medalCountSilver as medal_count_silver,
                pgd.medalCountGold as medal_count_gold,
                pgd.medalCountPlatinum as medal_count_platinum,
                pgd.medalCountDiamond as medal_count_diamond
            FROM players pd
                JOIN playerGameData pgd ON pgd.uuid = pd.uuid
                LEFT JOIN accounts a ON a.uuid = pd.uuid
            WHERE
                pgd.game = ?
                AND EXISTS (
                    SELECT cm.uuid FROM chatMessages cm
                    WHERE cm.uuid = pd.uuid
                        AND cm.game = pgd.game
                        AND cm.timestamp
                        BETWEEN ? AND ?
                        AND cm.partyId IS NULL
                        OR cm.partyId = ?
                )",
                state.config.game_name,
                first_timestamp,
                last_timestamp,
                party_id.0
            )
            .fetch(&state.database.pool)
        },
    );
    Ok(rows_stream
        .inspect_err(|e| tracing::error!("{e:?}"))
        .filter_map(async |x| x.ok())
        .map(|row| ChatPlayer {
            name: row.name,
            uuid: row.uuid,
            system_name: row.system_name,
            badge: row.badge,
            is_authenticated: row.is_authenticated != 0,
            rank: row.rank,
            medals: Medals(
                [
                    row.medal_count_bronze.unwrap_or_default(),
                    row.medal_count_silver.unwrap_or_default(),
                    row.medal_count_gold.unwrap_or_default(),
                    row.medal_count_platinum.unwrap_or_default(),
                    row.medal_count_diamond.unwrap_or_default(),
                ]
                .into(),
            ),
        })
        .collect()
        .await)
}

pub async fn delete_old_chat_messages(database: &Database) -> Result<()> {
    sqlx::query!(
        "DELETE FROM chatMessages WHERE timestamp < DATE_SUB(UTC_TIMESTAMP(), INTERVAL 1 DAY)"
    )
    .execute(&database.pool)
    .await?;
    Ok(())
}
