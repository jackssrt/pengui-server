use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc,
        nonpoison::{Mutex, RwLock},
    },
};

use crate::{
    client::Client,
    player::{
        Player,
        ids::{PlayerId, PlayerUuid},
    },
    session,
};
#[derive(Default)]
pub struct Players {
    pub players: Mutex<HashMap<PlayerUuid, Arc<RwLock<Player>>>>,
    pub free_ids: Mutex<VecDeque<PlayerId>>,
}
impl Players {
    pub fn get_next_free_id(
        players: &HashMap<PlayerUuid, Arc<RwLock<Player>>>,
        free_ids: &mut VecDeque<PlayerId>,
    ) -> PlayerId {
        free_ids
            .pop_front()
            // allocate a new id
            .unwrap_or(PlayerId(players.len()))
    }
    pub fn insert_new(
        players: &mut HashMap<PlayerUuid, Arc<RwLock<Player>>>,
        free_ids: &VecDeque<PlayerId>,
        player: Arc<RwLock<Player>>,
    ) -> Arc<RwLock<Player>> {
        players.insert(player.read().uuid.clone(), player.clone());
        player
    }
    pub fn remove_player(&self, player: Arc<RwLock<Player>>) {
        let (uuid, id) = player.with(|players| (player.read().uuid.clone(), player.read().id.0));
        tracing::debug!("removing player {} with id {}", uuid, id);
        self.players.lock().remove(&uuid);
        self.free_ids.lock().push_back(PlayerId(id));
        drop(player);
    }
    pub async fn get_by_uuid(&self, uuid: &PlayerUuid) -> Option<Arc<RwLock<Player>>> {
        let players = self.players.lock();
        players.get(uuid).cloned()
    }
    pub async fn broadcast_session_packet(&self, packet: session::client::packet::OutgoingPacket) {
        self.players
            .lock()
            .values()
            .map(|player| player.read().session_client.clone())
            .for_each(|session_client| {
                let packet = packet.clone();
                tokio::spawn(async move {
                    let _ = session_client.send_packet(packet).await;
                });
            });
    }
}
