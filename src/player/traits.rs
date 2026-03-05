use crate::{player::ids::PlayerUuid, server::state::AppState};
use anyhow::Result;

pub trait FetchForPlayerUuid: Sized {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Self>;
}
pub trait MaybeFetchForPlayerUuid: Sized {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Option<Self>>;
}
