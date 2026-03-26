use anyhow::Result;
use serde::Serialize;

use super::ids::PlayerUuid;
use crate::{player::traits::FetchForPlayerUuid, server::state::AppState};

#[derive(Debug, Clone, PartialEq, Eq, Default, PartialOrd, Ord, Serialize)]
#[repr(transparent)]
pub struct ScreenshotLimit(i32);
impl FetchForPlayerUuid for ScreenshotLimit {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Self> {
        Ok(sqlx::query!(
            "SELECT screenshotLimit FROM accounts WHERE uuid = ?",
            player_uuid.as_ref()
        )
        .fetch_optional(&state.database.pool)
        .await?
        .map(|record| Self(record.screenshotLimit))
        .unwrap_or_default())
    }
}
