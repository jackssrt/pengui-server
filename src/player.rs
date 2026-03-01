use anyhow::{Context, Result, bail};
use futures_util::TryStreamExt;
use std::{collections::HashSet, net::IpAddr, sync::Arc};

use crate::{
    party::ids::PartyUuid,
    player::{
        game_data::GameData,
        ids::{PlayerId, PlayerUuid},
        medal::Medals,
        moderation_status::ModerationStatus,
        privacy_settings::PrivacySettings,
        rank::Rank,
    },
    room::client::RoomClient,
    server::{players::Players, state::AppState},
    session::client::SessionClient,
};

pub mod disconnected;
pub mod friends;
pub mod game_data;
pub mod ids;
pub mod medal;
pub mod moderation_status;
pub mod privacy_settings;
pub mod rank;

/**
 * Data related to a player,
 * things that persist across sessions
 * see (`SessionClient`)[`crate::session::client::SessionClient`] for session locals
 */
pub struct Player {
    pub uuid: PlayerUuid,
    pub id: PlayerId,
    name: Option<String>,
    ip: IpAddr,
    rank: Rank,
    badge: Option<String>,
    medals: Medals,
    privacy_settings: PrivacySettings,
    moderation_status: ModerationStatus,
    party_id: Option<PartyUuid>,
    game_data: GameData,

    online_friends: HashSet<PlayerUuid>,
    blocked_users: HashSet<PlayerUuid>,
    // sockets
    room_client: Option<Arc<RoomClient>>,
    session_client: Arc<SessionClient>,
}

impl Player {
    pub async fn new(
        state: &AppState,
        session_client: SessionClient,
        ip: IpAddr,
        token: Option<&str>,
    ) -> Result<Arc<Self>> {
        let uuid;
        let name;
        let rank;
        let badge;
        let moderation_status;
        let medals;
        let party_id;
        let blocked_users;
        if let Some(token) = token && let Some(query) = sqlx::query!(
            "SELECT a.uuid, a.user, pd.rank, a.badge, pd.banned, pd.muted, pgd.medalCountBronze, pgd.medalCountSilver, pgd.medalCountGold, pgd.medalCountPlatinum, pgd.medalCountDiamond, pm.partyId FROM accounts a JOIN playerSessions ps ON ps.uuid = a.uuid JOIN players pd ON pd.uuid = a.uuid JOIN playerGameData pgd ON pgd.uuid = a.uuid LEFT JOIN partyMembers pm ON pm.uuid = a.uuid WHERE ps.sessionId = ? AND NOW() < ps.expiration",
            token
        ).fetch_optional(&state.database.pool).await? {
            uuid = PlayerUuid(query.uuid);
            name = Some(query.user);
            rank = Rank::from_repr(query.rank).context("rank value out of range in db")?;
            badge = query.badge;
            moderation_status = ModerationStatus::from_ints(query.banned, query.muted);
            medals = Medals([query.medalCountBronze, query.medalCountSilver, query.medalCountGold, query.medalCountPlatinum, query.medalCountDiamond]);
            party_id = query.partyId.map(PartyUuid);
            blocked_users = sqlx::query!("SELECT targetUuid FROM playerBlocks WHERE uuid = ?", uuid.0).fetch(&state.database.pool).map_ok(|x| PlayerUuid(x.targetUuid)).try_collect().await?;
        } else {
            name = None;
            rank = Rank::default();
            badge = None;
            medals = Medals::default();
            party_id = None;
            // get moderation status for the ip
            (uuid, moderation_status) = ModerationStatus::for_ip(state, ip).await?.unwrap_or_else(|| (PlayerUuid::new_random(), ModerationStatus::default()));
            blocked_users = HashSet::default();

            // create the player data
            sqlx::query!("INSERT INTO players (ip, uuid, banned) VALUES (?, ?, ?)", ip, uuid.0, false)
                .execute(&state.database.pool).await?;
        }
        {
            let players = state.players.players.lock();
            if players.values().filter(|x| x.ip == ip).count() > 3 {
                bail!("too many connections from ip");
            }
        }

        let mut ids_to_uuids = state.players.ids_to_uuids.lock();
        let id = Players::get_next_free_id(&ids_to_uuids);
        Ok(Players::insert_new(
            &mut state.players.players.lock(),
            &mut ids_to_uuids,
            Self {
                id,
                uuid,
                ip,
                name,
                rank,
                badge,
                moderation_status,
                medals,
                game_data: GameData::default(),
                room_client: None,
                session_client: Arc::new(session_client),
                party_id,
                blocked_users,
                online_friends: HashSet::default(),
                privacy_settings: PrivacySettings::default(),
            },
        ))
    }
    fn is_privated_to(&self, other: &Self) -> bool {
        let is_private_mode = self.privacy_settings.private || other.privacy_settings.private;
        let is_single_player_mode =
            self.privacy_settings.single_player || other.privacy_settings.single_player;
        let is_different_party = other.party_id.is_none() || self.party_id != other.party_id;
        let are_not_friends = !self.online_friends.contains(&other.uuid);
        (is_private_mode) && ((is_single_player_mode) || is_different_party && are_not_friends)
    }
    fn is_blocked_with(&self, other: &Self) -> bool {
        self.blocked_users.contains(&other.uuid) || other.blocked_users.contains(&other.uuid)
    }
    const fn is_unnamed_player_hidden_by(&self, other: &Self) -> bool {
        self.name.is_none() && other.privacy_settings.hide_unnamed_players
    }
}
