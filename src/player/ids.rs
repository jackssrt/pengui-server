use std::{fmt::Display, net::IpAddr, sync::Arc};

use anyhow::Result;
use rand::distr::{Alphanumeric, SampleString};
use serde::Serialize;

use crate::{server::state::AppState, traits::Random};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
#[repr(transparent)]
pub struct PlayerUuid(pub Arc<str>);
impl PlayerUuid {
    pub async fn fetch_for_token(state: &AppState, token: &str) -> Result<Option<Self>> {
        let query = sqlx::query!(
            "SELECT uuid FROM playerSessions WHERE sessionId = ? AND NOW() < expiration",
            token
        )
        .fetch_optional(&state.database.pool)
        .await?;
        Ok(query.map(|x| Self(x.uuid.into())))
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
            Self(query.uuid.into())
        } else {
            let uuid = Self::random();
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
}
impl Default for PlayerUuid {
    fn default() -> Self {
        Self("0000000000000000".into())
    }
}
impl Random for PlayerUuid {
    fn random() -> Self {
        Self(Alphanumeric.sample_string(&mut rand::rng(), 16).into())
    }
}
impl Display for PlayerUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Serialize for PlayerUuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        AsRef::<str>::as_ref(&self.0).serialize(serializer)
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Copy, Debug, Default)]
#[repr(transparent)]
// these are 0-index everywhere. i checked.
pub struct PlayerId(pub usize);
