use std::collections::HashSet;

use anyhow::Result;
use futures_util::{StreamExt, TryStreamExt};

use super::{ids::PlayerUuid, traits::FetchForPlayerUuid};
use crate::server::state::AppState;

pub struct BlockedUsers(pub HashSet<PlayerUuid>);
impl BlockedUsers {
    pub fn contains(&self, player_uuid: &PlayerUuid) -> bool {
        self.0.contains(player_uuid)
    }
}
impl FetchForPlayerUuid for BlockedUsers {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Self> {
        Ok(Self(
            sqlx::query!(
                "SELECT targetUuid FROM playerBlocks where uuid = ?",
                player_uuid.as_ref()
            )
            .fetch(&state.database.pool)
            .map(|x| x.map(|x| PlayerUuid(x.targetUuid.into())))
            .try_collect()
            .await?,
        ))
    }
}
