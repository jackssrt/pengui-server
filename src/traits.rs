use anyhow::Result;

use crate::{player::ids::PlayerUuid, server::state::AppState};

pub trait Random: Sized {
    fn random() -> Self;
}

pub trait FetchForPlayerUuid: Sized {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Self>;
}
pub trait MaybeFetchForPlayerUuid: Sized {
    async fn fetch_for_player_uuid(
        state: &AppState,
        player_uuid: &PlayerUuid,
    ) -> Result<Option<Self>>;
}
