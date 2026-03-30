use anyhow::Result;

use crate::{player::ids::PlayerUuid, server::state::AppState, traits::FetchForPlayerUuid};

#[derive(Default, strum::EnumIs)]
pub enum ModerationStatus {
    #[default]
    Clear,
    Muted,
    Banned,
}
impl ModerationStatus {
    pub const fn from_booleans(is_banned: bool, is_muted: bool) -> Self {
        match (is_banned, is_muted) {
            (true, ..) => Self::Banned,
            (.., true) => Self::Muted,
            _ => Self::Clear,
        }
    }
    pub const fn from_ints(banned: i8, muted: i8) -> Self {
        Self::from_booleans(banned != 0, muted != 0)
    }
}
impl FetchForPlayerUuid for ModerationStatus {
    async fn fetch_for_player_uuid(state: &AppState, uuid: &PlayerUuid) -> Result<Self> {
        Ok(sqlx::query!(
            "SELECT muted, banned FROM players WHERE uuid = ?",
            uuid.0.as_ref()
        )
        .fetch_optional(&state.database.pool)
        .await?
        .map(|record| Self::from_ints(record.banned, record.muted))
        .unwrap_or_default())
    }
}
