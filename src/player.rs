use std::{
    net::IpAddr,
    sync::{Arc, nonpoison::RwLock},
};

use anyhow::{Result, bail};
use tokio::sync::mpsc;

use crate::{
    party::ids::PartyId,
    player::{
        badge::BadgeName,
        game_data::GameData,
        ids::{PlayerId, PlayerUuid},
        medal::Medals,
        moderation_status::ModerationStatus,
        name::PlayerName,
        privacy_settings::PrivacySettings,
        rank::Rank,
        relations::Relations,
        traits::{FetchForPlayerUuid, MaybeFetchForPlayerUuid},
    },
    room::client::RoomClient,
    server::state::AppState,
    session::{self, client::SessionClient},
};

pub mod badge;
pub mod badge_slots;
pub mod disconnected;
pub mod game_data;
pub mod ids;
pub mod locations;
pub mod medal;
pub mod moderation_status;
pub mod name;
pub mod privacy_settings;
pub mod rank;
pub mod relations;
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
    pub relations: Relations,

    // sockets
    pub room_client: Option<Arc<RoomClient>>,
    pub session_client: Arc<SessionClient>,
}

impl Player {
    pub async fn new(
        state: &'static AppState,
        is_authenticated: bool,
        uuid: PlayerUuid,
        ip: IpAddr,
        session_outgoing_sender: mpsc::UnboundedSender<session::client::packet::OutgoingPacket>,
    ) -> Result<Arc<RwLock<Self>>> {
        // all this data is fetched here to avoid locking the players and ids_to_uuids mutexes for too long
        let name = PlayerName::fetch_for_player_uuid(state, &uuid).await?;
        let rank = Rank::fetch_for_player_uuid(state, &uuid).await?;
        let badge = BadgeName::fetch_for_player_uuid(state, &uuid).await?;
        let moderation_status = ModerationStatus::fetch_for_player_uuid(state, &uuid).await?;
        let medals = Medals::fetch_for_player_uuid(state, &uuid).await?;
        let party_id = PartyId::fetch_for_player_uuid(state, &uuid).await?;
        let relations = Relations::fetch_for_player_uuid(state, &uuid).await?;
        let game_data = GameData::default();
        let privacy_settings = PrivacySettings::default();
        {
            // the limit is 4 per ip
            if state
                .players
                .players
                .iter()
                .filter(|x| x.read().ip == ip)
                .count()
                >= 4
            {
                bail!("too many connections from ip");
            }

            let mut free_ids = state.players.free_ids.lock();
            let id = state.players.get_next_free_id(&mut free_ids);
            Ok(state.players.insert_new(
                &free_ids,
                Arc::new_cyclic(|weak| {
                    RwLock::new(Self {
                        id,
                        uuid,
                        ip,
                        name,
                        rank,
                        badge,
                        moderation_status,
                        medals,
                        room_client: None,
                        session_client: Arc::new(SessionClient::new(
                            state,
                            weak.clone(),
                            session_outgoing_sender,
                        )),
                        party_id,
                        relations,
                        privacy_settings,
                        game_data,
                        is_authenticated,
                    })
                }),
            ))
        }
    }
    pub async fn update_player_game_data(
        state: &'static AppState,
        uuid: &PlayerUuid,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO playerGameData (uuid, game, online) VALUES (?, ?, 1) ON DUPLICATE KEY UPDATE online = 1, timestampLastActive = UTC_TIMESTAMP()",
            uuid.0,
            state.config.game_name
        ).execute(&state.database.pool).await?;
        Ok(())
    }
    pub fn is_privated_to(&self, other: &Self) -> bool {
        let PrivacySettings {
            private,
            singleplayer,
            ..
        } = self.privacy_settings.or(&other.privacy_settings);
        let is_different_party = other.party_id.is_none() || self.party_id != other.party_id;
        let are_not_friends = !self.relations.online_friends.contains(&other.uuid);
        (private) && ((singleplayer) || is_different_party && are_not_friends)
    }
    pub fn is_blocked_with(&self, other: &Self) -> bool {
        self.relations.blocked_users.contains(&other.uuid)
            || other.relations.blocked_users.contains(&other.uuid)
    }
    pub const fn is_unnamed_player_hidden_by(&self, other: &Self) -> bool {
        self.name.is_none() && other.privacy_settings.hide_unnamed_players
    }
}
