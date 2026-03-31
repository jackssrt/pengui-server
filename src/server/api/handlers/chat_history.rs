use std::sync::Arc;

use anyhow::{Result, anyhow};
use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    chat::{history::get_chat_history, ids::MessageId}, party::ids::PartyId, player::traits::MaybeFetchForPlayerUuid, server::{
        api::extractors::authentication::OptionalHeaderAuthentication, error::AppError,
        state::AppState,
    }
};
#[inline]
const fn global_message_limit_default() -> u8 {
    100
}

#[inline]
const fn party_message_limit_default() -> u8 {
    250
}

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(rename = "lastMsgId")]
    last_message_id: Option<Arc<str>>,
    #[serde(
        rename = "globalMsgLimitParam",
        default = "global_message_limit_default"
    )]
    global_message_limit: u8,
    #[serde(rename = "partyMsgLimitParam", default = "party_message_limit_default")]
    party_message_limit: u8,
}

pub fn validate_message_id(
    last_message_id: Option<Arc<str>>,
) -> Result<Option<MessageId>, AppError> {
    if let Some(ref last_message_id) = last_message_id
        && last_message_id.len() != 12
    {
        // can't use bail! here cause i need to convert the anyhow error into an AppError
        Err(anyhow!("invalid lastMsgId"))?;
    }
    // last_message_id is "valid"
    let last_message_id = last_message_id.map(MessageId);
    Ok(last_message_id)
}

#[axum::debug_handler]
pub async fn handle_chat_history(
    State(state): State<&'static AppState>,
    Query(QueryParams {
        last_message_id,
        global_message_limit,
        party_message_limit,
    }): Query<QueryParams>,
    OptionalHeaderAuthentication(auth): OptionalHeaderAuthentication,
) -> Result<impl IntoResponse, AppError> {
    let global_message_limit = global_message_limit.min(global_message_limit_default());
    let party_message_limit = party_message_limit.min(party_message_limit_default());
    let last_message_id = validate_message_id(last_message_id)?;

    let player_uuid = auth.take_uuid();
    let party_id = PartyId::fetch_for_player_uuid(state, &player_uuid).await?;
    let history = get_chat_history(
        state,
        party_id,
        global_message_limit,
        party_message_limit,
        last_message_id,
    )
    .await?;
    Ok(Json(history))
}

#[derive(Deserialize)]
pub struct ClearChatHistoryQueryParams {
    #[serde(rename = "lastGlobalMsgId")]
    last_global_message_id: Option<Arc<str>>,
    #[serde(rename = "lastPartyMsgId")]
    last_party_message_id: Option<Arc<str>>,
}

#[axum::debug_handler]
pub async fn handle_clear_chat_history(
    State(state): State<&'static AppState>,
    Query(ClearChatHistoryQueryParams {
        last_global_message_id,
        last_party_message_id,
    }): Query<ClearChatHistoryQueryParams>,
    OptionalHeaderAuthentication(auth): OptionalHeaderAuthentication,
) -> Result<&'static str, AppError> {
    let player_uuid = auth.take_uuid();

    if let Some(last_global_message_id) = validate_message_id(last_global_message_id)? {
        sqlx::query!(
            "UPDATE playerGameData SET lastGlobalMsgId = ? WHERE uuid = ? AND game = ?",
            last_global_message_id.0.as_ref(),
            player_uuid.0.as_ref(),
            state.config.game_name,
        )
        .execute(&state.database.pool)
        .await?;
    }
    if let Some(last_party_message_id) = validate_message_id(last_party_message_id)? {
        sqlx::query!(
            "UPDATE playerGameData SET lastPartyMsgId = ? WHERE uuid = ? AND game = ?",
            last_party_message_id.0.as_ref(),
            player_uuid.0.as_ref(),
            state.config.game_name,
        )
        .execute(&state.database.pool)
        .await?;
    }

    Ok("ok")
}
