use std::net::IpAddr;

use crate::{
    player::{ids::PlayerUuid, moderation_status::ModerationStatus},
    server::config::Config,
};
use anyhow::Result;
use sqlx::mysql::MySqlPool;

pub struct Database {
    pub pool: MySqlPool,
}
impl Database {
    pub async fn connect(config: &Config) -> Result<Self> {
        let url = &format!(
            "mysql://{}:{}@{}/{}",
            config.db_user, config.db_user, config.db_addr, config.db_name
        );
        Ok(Self {
            pool: MySqlPool::connect(url).await?,
        })
    }
    pub async fn run_start_up_stuff(&self, config: &Config) -> Result<()> {
        // set all previously online players to offline in playerGameData
        sqlx::query!(
            "UPDATE playerGameData SET online = 0 WHERE game = ? and online = 1",
            config.game_name
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn get_player_data_for_ip(
        &self,
        ip: &IpAddr,
    ) -> Result<Option<(PlayerUuid, ModerationStatus)>> {
        let query = sqlx::query!("SELECT uuid, banned, muted FROM players WHERE ip = ?", ip)
            .fetch_optional(&self.pool)
            .await?;
        Ok(query.map(|query| {
            (
                PlayerUuid(query.uuid),
                ModerationStatus::from_ints(query.banned, query.muted),
            )
        }))
    }
    pub async fn get_player_data_for_token(
        &self,
        token: &str,
    ) -> Result<Option<(PlayerUuid, ModerationStatus)>> {
        let query = sqlx::query!("SELECT p.uuid, p.banned, p.muted FROM players p JOIN playerSessions ps ON ps.uuid = p.uuid WHERE ps.sessionId = ? AND NOW() < ps.expiration", token).fetch_optional(&self.pool).await?;
        Ok(query.map(|query| {
            (
                PlayerUuid(query.uuid),
                ModerationStatus::from_ints(query.banned, query.muted),
            )
        }))
    }
}
