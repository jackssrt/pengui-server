use std::net::IpAddr;

use crate::{
    player::{ids::PlayerUuid, moderation_status::ModerationStatus, rank::Rank},
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
}
