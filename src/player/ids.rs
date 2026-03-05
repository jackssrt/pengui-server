use std::{fmt::Display, net::IpAddr};

use anyhow::Result;
use futures_util::future::OptionFuture;
use rand::distr::{Alphanumeric, SampleString};
use serde::Serialize;

use crate::server::state::AppState;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize)]
#[repr(transparent)]
pub struct PlayerUuid(pub String);
impl PlayerUuid {
    pub fn new_random() -> Self {
        // rand::rng() returns a ThreadRng
        // which is cryptographically secure
        // according to the rand crate docs
        Self(Alphanumeric.sample_string(&mut rand::rng(), 16))
    }
    pub async fn fetch_for_token(state: &AppState, token: &str) -> Result<Option<Self>> {
        let query = sqlx::query!(
            "SELECT uuid FROM playerSessions WHERE sessionId = ? AND NOW() < expiration",
            token
        )
        .fetch_optional(&state.database.pool)
        .await?;
        Ok(query.map(|x| Self(x.uuid)))
    }
    pub async fn fetch_for_ip(state: &AppState, ip: &IpAddr) -> Result<Self> {
        // ip is already a unique key
        // for some reason... do they not know about NAT?
        // and the fact that they also have code that allows
        // up to 4 connections from the same ip??
        // maybe the schema i found is just too old
        let query = sqlx::query!("SELECT uuid FROM players WHERE ip = ?", ip)
            .fetch_optional(&state.database.pool)
            .await?;
        Ok(if let Some(query) = query {
            Self(query.uuid)
        } else {
            let uuid = Self::new_random();
            sqlx::query!(
                "INSERT INTO players (ip, uuid, banned) VALUES (?, ?, ?)",
                ip,
                uuid.0,
                false
            )
            .execute(&state.database.pool)
            .await?;
            uuid
        })
    }
    pub async fn fetch_for_token_or_ip(
        state: &AppState,
        token: Option<&str>,
        ip: &IpAddr,
    ) -> Result<(bool, Self)> {
        Ok(
            match OptionFuture::from(token.map(|token| Self::fetch_for_token(state, token)))
                .await
                .transpose()?
                .flatten()
            {
                Some(uuid) => (true, uuid),
                None => (false, Self::fetch_for_ip(state, ip).await?),
            },
        )
    }
}
impl Default for PlayerUuid {
    fn default() -> Self {
        Self::new_random()
    }
}
impl Display for PlayerUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Serialize)]
#[repr(transparent)]
pub struct PlayerId(pub usize);
