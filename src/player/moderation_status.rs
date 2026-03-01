use std::net::IpAddr;

use anyhow::Result;

use crate::{player::ids::PlayerUuid, server::state::AppState};

#[derive(Default)]
pub enum ModerationStatus {
    #[default]
    None,
    Muted,
    Banned,
}
impl ModerationStatus {
    pub const fn from_booleans(is_banned: bool, is_muted: bool) -> Self {
        match (is_banned, is_muted) {
            (true, ..) => Self::Banned,
            (.., true) => Self::Muted,
            _ => Self::None,
        }
    }
    pub const fn from_ints(banned: i8, muted: i8) -> Self {
        Self::from_booleans(banned != 0, muted != 0)
    }
    pub async fn for_ip(state: &AppState, ip: IpAddr) -> Result<Option<(PlayerUuid, Self)>> {
        let query = sqlx::query!("SELECT uuid, banned, muted FROM players WHERE ip = ?", ip)
            .fetch_optional(&state.database.pool)
            .await?;
        Ok(query.map(|query| {
            (
                PlayerUuid(query.uuid),
                Self::from_ints(query.banned, query.muted),
            )
        }))
    }
}
