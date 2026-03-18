use anyhow::Result;
use serde::Serialize;

use super::ids::PlayerUuid;
use crate::{player::traits::MaybeFetchForPlayerUuid, server::state::AppState};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Default)]
#[repr(transparent)]
pub struct BadgeName(pub String);

impl MaybeFetchForPlayerUuid for BadgeName {
    async fn fetch_for_player_uuid(
        state: &AppState,
        player_uuid: &PlayerUuid,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query!("SELECT badge FROM accounts WHERE uuid = ?", player_uuid.0)
                .fetch_optional(&state.database.pool)
                .await?
                .map(|x| Self(x.badge)),
        )
    }
}
