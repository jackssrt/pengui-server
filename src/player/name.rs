use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;

use super::{ids::PlayerUuid, traits::MaybeFetchForPlayerUuid};
use crate::server::state::AppState;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Default)]
#[repr(transparent)]
pub struct PlayerName(pub Arc<str>);

impl MaybeFetchForPlayerUuid for PlayerName {
    async fn fetch_for_player_uuid(
        state: &AppState,
        player_uuid: &PlayerUuid,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_scalar!("SELECT user FROM accounts WHERE uuid = ?", player_uuid.0,)
                .fetch_optional(&state.database.pool)
                .await?
                .map(Into::into)
                .map(Self),
        )
    }
}
