use std::{
    collections::HashSet,
    net::IpAddr,
    sync::{Arc, nonpoison::RwLock},
};

use anyhow::{Result, bail};

use crate::{
    party::ids::PartyId,
    player::{
        badge::BadgeName,
        blocked_users::BlockedUsers,
        game_data::GameData,
        ids::{PlayerId, PlayerUuid},
        medal::Medals,
        moderation_status::ModerationStatus,
        name::PlayerName,
        privacy_settings::PrivacySettings,
        rank::Rank,
        traits::{FetchForPlayerUuid, MaybeFetchForPlayerUuid},
    },
    room::client::RoomClient,
    server::{players::Players, state::AppState},
    session::client::SessionClient,
};

pub mod badge;
pub mod badge_slots;
pub mod blocked_users;
pub mod disconnected;
pub mod friends;
pub mod game_data;
pub mod ids;
pub mod medal;
pub mod moderation_status;
pub mod name;
pub mod privacy_settings;
pub mod rank;
pub mod screenshot_limit;
pub mod traits;

/**
 * Data related to a player,
 * things that persist across sessions
 * see (`SessionClient`)[`crate::session::client::SessionClient`] for session locals
 */
pub struct Player {
    pub uuid: PlayerUuid,
    pub id: PlayerId,
    pub name: Option<PlayerName>,
    pub ip: IpAddr,
    pub rank: Rank,
    pub badge: Option<BadgeName>,
    pub medals: Medals,
    pub privacy_settings: PrivacySettings,
    pub moderation_status: ModerationStatus,
    pub party_id: Option<PartyId>,
    pub game_data: GameData,
    pub is_authenticated: bool,

    pub online_friends: HashSet<PlayerUuid>,
    pub blocked_users: BlockedUsers,
    // sockets
    pub room_client: Option<Arc<RoomClient>>,
    pub session_client: Arc<SessionClient>,
}

impl Player {
    pub async fn new(
        state: &AppState,
        session_client: SessionClient,
        is_authenticated: bool,
        uuid: PlayerUuid,
        ip: IpAddr,
    ) -> Result<Arc<RwLock<Self>>> {
        // all this data is fetched here to avoid locking the players and ids_to_uuids mutexes for too long
        let name = PlayerName::fetch_for_player_uuid(state, &uuid).await?;
        let rank = Rank::fetch_for_player_uuid(state, &uuid).await?;
        let badge = BadgeName::fetch_for_player_uuid(state, &uuid).await?;
        let moderation_status = ModerationStatus::fetch_for_player_uuid(state, &uuid).await?;
        let medals = Medals::fetch_for_player_uuid(state, &uuid).await?;
        let party_id = PartyId::fetch_for_player_uuid(state, &uuid).await?;
        let blocked_users = BlockedUsers::fetch_for_player_uuid(state, &uuid).await?;
        let online_friends = HashSet::default();
        let game_data = GameData::default();
        let privacy_settings = PrivacySettings::default();
        let session_client = Arc::new(session_client);
        {
            // get the mutable lock here to prevent a time-of-check to time-of-use race condition where
            // someone could connect a ton of clients from the same ip
            let mut players = state.players.players.lock();

            // the limit is 4 per ip
            if players.values().filter(|x| x.read().ip == ip).count() >= 4 {
                bail!("too many connections from ip");
            }

            let mut ids_to_uuids = state.players.ids_to_uuids.lock();
            let id = Players::get_next_free_id(&ids_to_uuids);
            Ok(Players::insert_new(
                &mut players,
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
                    room_client: None,
                    session_client,
                    party_id,
                    blocked_users,
                    online_friends,
                    privacy_settings,
                    game_data,
                    is_authenticated,
                },
            ))
        }
    }
    pub fn is_privated_to(&self, other: &Self) -> bool {
        let PrivacySettings {
            private,
            single_player,
            ..
        } = self.privacy_settings.or(&other.privacy_settings);
        let is_different_party = other.party_id.is_none() || self.party_id != other.party_id;
        let are_not_friends = !self.online_friends.contains(&other.uuid);
        (private) && ((single_player) || is_different_party && are_not_friends)
    }
    pub fn is_blocked_with(&self, other: &Self) -> bool {
        self.blocked_users.contains(&other.uuid) || other.blocked_users.contains(&other.uuid)
    }
    pub const fn is_unnamed_player_hidden_by(&self, other: &Self) -> bool {
        self.name.is_none() && other.privacy_settings.hide_unnamed_players
    }
}
