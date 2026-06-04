use std::collections::HashSet;

use crate::{
    player::{ids::PlayerUuid, traits::FetchForPlayerUuid},
    server::state::AppState,
};

pub mod blocked_users;

use anyhow::Result;
pub use blocked_users::BlockedUsers;

pub struct Relations {
    pub online_friends: HashSet<PlayerUuid>,
    pub blocked_users: BlockedUsers,
}

impl FetchForPlayerUuid for Relations {
    async fn fetch_for_player_uuid(state: &AppState, uuid: &PlayerUuid) -> Result<Self> {
        Ok(Self {
            online_friends: HashSet::new(),
            blocked_users: BlockedUsers::fetch_for_player_uuid(state, uuid).await?,
        })
    }
}
