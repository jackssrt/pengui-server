use anyhow::Result;
use serde::Serialize;

use crate::{player::traits::FetchForPlayerUuid, server::state::AppState};

use super::ids::PlayerUuid;

#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct BadgeSlots {
    #[serde(rename = "badgeSlotRows")]
    rows: i32,
    #[serde(rename = "badgeSlotCols")]
    columns: i32,
}
impl Default for BadgeSlots {
    fn default() -> Self {
        Self {
            rows: 1,
            columns: 3,
        }
    }
}

impl FetchForPlayerUuid for BadgeSlots {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Self> {
        let record = sqlx::query!(
            "SELECT badgeSlotRows, badgeSlotCols FROM accounts WHERE uuid = ?",
            player_uuid.0
        )
        .fetch_one(&state.database.pool)
        .await?;

        Ok(Self {
            rows: record.badgeSlotRows,
            columns: record.badgeSlotCols,
        })
    }
}
