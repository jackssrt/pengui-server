use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;

use super::ids::PlayerUuid;
use crate::{server::state::AppState, traits::MaybeFetchForPlayerUuid};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Default)]
#[repr(transparent)]
pub struct BadgeName(pub Arc<str>);

impl MaybeFetchForPlayerUuid for BadgeName {
    async fn fetch_for_player_uuid(
        state: &AppState,
        player_uuid: &PlayerUuid,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query!(
            "SELECT badge FROM accounts WHERE uuid = ?",
            player_uuid.as_ref()
        )
        .fetch_optional(&state.database.pool)
        .await?
        .map(|x| Self(x.badge.into())))
    }
}
