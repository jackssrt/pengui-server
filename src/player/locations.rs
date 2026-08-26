use futures_util::{StreamExt, TryStreamExt};
use serde::Serialize;

use crate::{
    player::{ids::PlayerUuid, locations::ids::LocationId, traits::FetchForPlayerUuid},
    server::state::AppState,
};

pub mod ids;
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize)]
#[repr(transparent)]
pub struct Locations(pub Vec<LocationId>);

impl FetchForPlayerUuid for Locations {
    async fn fetch_for_player_uuid(
        state: &AppState,
        player_uuid: &PlayerUuid,
    ) -> anyhow::Result<Self> {
        Ok(Self(sqlx::query_scalar!(
            "SELECT gl.id FROM playerGameLocations pgl JOIN gameLocations gl ON gl.id = pgl.locationId AND gl.game = ? WHERE pgl.uuid = ?",
            &state.config.game_name,
            player_uuid.0,
        ).fetch(&state.database.pool).map(|result| result.map(LocationId)).try_collect().await?))
    }
}
