use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    player::{ids::PlayerUuid, traits::MaybeFetchForPlayerUuid},
    server::state::AppState,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug, Deserialize)]
#[repr(transparent)]
pub struct PartyId(pub i32);

impl MaybeFetchForPlayerUuid for PartyId {
    async fn fetch_for_player_uuid(
        state: &AppState,
        player_uuid: &PlayerUuid,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_scalar!(
            "SELECT partyId FROM partyMembers where uuid = ?",
            player_uuid.0
        )
        .fetch_optional(&state.database.pool)
        .await?
        .map(Self))
    }
}
