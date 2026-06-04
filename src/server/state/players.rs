use std::{
    collections::VecDeque,
    sync::{
        Arc,
        nonpoison::{Mutex, RwLock},
    },
};

use dashmap::DashMap;

use crate::{
    chat::ids::MessageId,
    client::Client,
    player::{
        Player,
        ids::{PlayerId, PlayerUuid},
        medal::Medals,
        name::PlayerName,
        rank::Rank,
    },
    session,
    traits::Random,
};
#[derive(Default)]
pub struct Players {
    pub players: DashMap<PlayerUuid, Arc<RwLock<Player>>>,
    pub free_ids: Mutex<VecDeque<PlayerId>>,
}
impl Players {
    pub fn get_next_free_id(&self, free_ids: &mut VecDeque<PlayerId>) -> PlayerId {
        free_ids
            .pop_front()
            // allocate a new id
            .unwrap_or(PlayerId(self.players.len()))
    }
    pub fn insert_new(
        &self,
        free_ids: &VecDeque<PlayerId>,
        player: Arc<RwLock<Player>>,
    ) -> Arc<RwLock<Player>> {
        self.players
            .insert(player.read().uuid.clone(), player.clone());
        player
    }
    pub fn remove_player(&self, player: Arc<RwLock<Player>>) {
        let (uuid, id) = player.with(|players| (player.read().uuid.clone(), player.read().id.0));
        tracing::debug!("removing player {} with id {}", uuid, id);
        self.players.remove(&uuid);
        self.free_ids.lock().push_back(PlayerId(id));
        drop(player);
    }
    pub async fn get_by_uuid(&self, uuid: &PlayerUuid) -> Option<Arc<RwLock<Player>>> {
        self.players.get(uuid).map(|x| x.value().clone())
    }
    pub async fn broadcast_session_packet(&self, packet: session::client::packet::OutgoingPacket) {
        self.players
            .iter()
            .map(|player| player.read().session_client.clone())
            .for_each(|session_client| {
                let packet = packet.clone();
                tokio::spawn(async move {
                    let _ = session_client.send_packet(packet).await;
                });
            });
    }
    pub async fn broadcast_system_message(&self, message: Arc<str>) {
        let uuid = PlayerUuid::default();
        self.broadcast_session_packet(session::client::packet::OutgoingPacket::PlayerInfo {
            uuid: uuid.clone(),
            name: PlayerName("YNO".into()),
            system: Arc::default(),
            rank: Rank::Developer,
            is_authenticated: true,
            badge: None,
            medals: Medals::default(),
        })
        .await;
        self.broadcast_session_packet({
            session::client::packet::OutgoingPacket::SayGlobal {
                uuid,
                map_id: 0,
                previous_map_id: 0,
                previous_locations: Arc::default(),
                x: 0,
                y: 0,
                content: message,
                message_id: MessageId::random(),
            }
        })
        .await;
    }
}
