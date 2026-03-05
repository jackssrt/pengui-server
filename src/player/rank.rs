use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use strum::FromRepr;

use crate::{
    player::{ids::PlayerUuid, traits::FetchForPlayerUuid},
    server::state::AppState,
};

#[derive(Deserialize, Serialize, FromRepr, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum Rank {
    #[default]
    User,
    Moderator,
    Developer,
}
impl FetchForPlayerUuid for Rank {
    async fn fetch_for_player_uuid(state: &AppState, uuid: &PlayerUuid) -> Result<Self> {
        Ok(
            sqlx::query!("SELECT `rank` FROM players WHERE uuid = ?", uuid.0)
                .fetch_optional(&state.database.pool)
                .await?
                .map(|x| {
                    Self::from_repr(x.rank.try_into()?)
                        .ok_or_else(|| anyhow!("rank value in db out of range"))
                })
                .transpose()?
                .unwrap_or_default(),
        )
    }
}
