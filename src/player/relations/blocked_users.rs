use std::collections::HashSet;

use anyhow::Result;
use futures_util::{StreamExt, TryStreamExt};

use crate::{
    player::{ids::PlayerUuid, traits::FetchForPlayerUuid},
    server::state::AppState,
};

pub struct BlockedUsers(pub HashSet<PlayerUuid>);
impl BlockedUsers {
    pub fn contains(&self, player_uuid: &PlayerUuid) -> bool {
        self.0.contains(player_uuid)
    }
}
impl FetchForPlayerUuid for BlockedUsers {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Self> {
        Ok(Self(
            sqlx::query_scalar!(
                "SELECT targetUuid FROM playerBlocks where uuid = ?",
                player_uuid.0
            )
            .fetch(&state.database.pool)
            .map(|x| x.map(Into::into).map(PlayerUuid))
            .try_collect()
            .await?,
        ))
    }
}
