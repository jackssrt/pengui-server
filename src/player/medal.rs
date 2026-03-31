use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;
use strum::FromRepr;

use super::{ids::PlayerUuid, traits::FetchForPlayerUuid};
use crate::server::state::AppState;

#[derive(FromRepr)]
#[repr(u8)]
pub enum Medal {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

// index with Medal
#[derive(Default, Serialize, Clone, Debug)]
pub struct Medals(pub Arc<[i8; 5]>);

impl FetchForPlayerUuid for Medals {
    async fn fetch_for_player_uuid(state: &AppState, player_uuid: &PlayerUuid) -> Result<Self> {
        let query = sqlx::query!(
            "SELECT medalCountBronze, medalCountSilver, medalCountGold, medalCountPlatinum, medalCountDiamond FROM playerGameData WHERE uuid = ?",
            player_uuid.as_ref()
        ).fetch_optional(&state.database.pool).await?;

        Ok(query.map_or_else(Self::default, |query| {
            Self(Arc::new(
                ([
                    query.medalCountBronze,
                    query.medalCountSilver,
                    query.medalCountGold,
                    query.medalCountPlatinum,
                    query.medalCountDiamond,
                ])
                .map(|x| x.unwrap_or(0)),
            ))
        }))
    }
}
