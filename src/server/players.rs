use std::{
    collections::HashMap,
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
    /// None for holes in vec
    pub ids_to_uuids: Mutex<Vec<Option<PlayerUuid>>>,
}
impl Players {
    pub fn get_next_free_id(ids_to_uuids: &[Option<PlayerUuid>]) -> PlayerId {
        ids_to_uuids
            .iter()
            .enumerate()
            // find the first None that we can reuse
            .find_map(|(i, x)| match x {
                None => Some(PlayerId(i)),
                Some(_) => None,
            })
            // allocate a new id
            .unwrap_or_else(|| PlayerId(ids_to_uuids.len() + 1))
    }
    pub fn insert_new(
        players: &mut HashMap<PlayerUuid, Arc<RwLock<Player>>>,
        ids_to_uuids: &mut Vec<Option<PlayerUuid>>,
        player: Arc<RwLock<Player>>,
    ) -> Arc<RwLock<Player>> {
        let (uuid, id) = player.with(|player| (player.uuid.clone(), player.id.0));
        players.insert(uuid.clone(), player.clone());
        if let Some(x) = ids_to_uuids.get_mut(id) {
            *x = Some(uuid);
        } else {
            // vec is too short
            ids_to_uuids.extend_one(Some(uuid));
        }
        player
    }
    pub fn remove_player(&self, player: Arc<RwLock<Player>>) {
        // -1 because we own an Arc to the player that we're removing
        tracing::debug!(
            "removing player {} with id {} rc {}",
            player.read().uuid.0,
            player.read().id.0,
            Arc::strong_count(&player) - 1
        );
        debug_assert!(Arc::strong_count(&player) == 1);
        let id = player.read().id.0;
        self.players.lock().remove(&player.read().uuid);
        if let Some(x) = self.ids_to_uuids.lock().get_mut(id) {
            *x = None;
        }
        drop(player);
    }
    pub async fn get_by_uuid(&self, uuid: &PlayerUuid) -> Option<Arc<RwLock<Player>>> {
        let players = self.players.lock();
        players.get(uuid).cloned()
    }
    pub async fn get_by_id(&self, id: &PlayerId) -> Option<Arc<RwLock<Player>>> {
        let uuids = self.ids_to_uuids.lock();
        let uuid = uuids.get(id.0)?.as_ref()?;
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
