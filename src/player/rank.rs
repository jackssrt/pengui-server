use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use strum::{EnumIs, FromRepr};

use super::{ids::PlayerUuid, traits::FetchForPlayerUuid};
use crate::server::state::AppState;

#[derive(
    Serialize, Deserialize, FromRepr, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Debug, EnumIs,
)]
#[repr(u8)]
pub enum Rank {
    #[default]
    #[serde(rename = "0")]
    User,
    #[serde(rename = "1")]
    Moderator,
    #[serde(rename = "2")]
    Developer,
}
impl FetchForPlayerUuid for Rank {
    async fn fetch_for_player_uuid(state: &AppState, uuid: &PlayerUuid) -> Result<Self> {
        Ok(
            sqlx::query_scalar!("SELECT `rank` FROM players WHERE uuid = ?", uuid.0)
                .fetch_optional(&state.database.pool)
                .await?
                .map(|rank| {
                    Self::from_repr(rank.try_into()?)
                        .ok_or_else(|| anyhow!("rank value in db out of range"))
                })
                .transpose()?
                .unwrap_or_default(),
        )
    }
}
