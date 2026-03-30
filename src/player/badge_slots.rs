use anyhow::Result;
use serde::Serialize;

use super::ids::PlayerUuid;
use crate::{server::state::AppState, traits::FetchForPlayerUuid};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Default)]
pub struct BadgeSlots {
    #[serde(rename = "badgeSlotRows")]
    rows: i32,
    #[serde(rename = "badgeSlotCols")]
    columns: i32,
}

impl FetchForPlayerUuid for BadgeSlots {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Self> {
        let record = sqlx::query!(
            "SELECT badgeSlotRows, badgeSlotCols FROM accounts WHERE uuid = ?",
            player_uuid.as_ref()
        )
        .fetch_optional(&state.database.pool)
        .await?;

        Ok(record.map_or_else(Self::default, |record| Self {
            rows: record.badgeSlotRows,
            columns: record.badgeSlotCols,
        }))
    }
}
