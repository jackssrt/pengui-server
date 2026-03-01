use std::{
    collections::HashMap,
    sync::{Arc, nonpoison::Mutex},
};

use crate::player::{
    Player,
    ids::{PlayerId, PlayerUuid},
};
#[derive(Default)]
pub struct Players {
    pub players: Mutex<HashMap<PlayerUuid, Arc<Player>>>,
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
            .unwrap_or(PlayerId(ids_to_uuids.len()))
    }
    pub fn insert_new(
        players: &mut HashMap<PlayerUuid, Arc<Player>>,
        ids_to_uuids: &mut [Option<PlayerUuid>],
        player: Player,
    ) -> Arc<Player> {
        let player = Arc::new(player);
        players.insert(player.uuid.clone(), player.clone());
        ids_to_uuids[player.id.0] = Some(player.uuid.clone());
        player
    }
    pub fn get_by_uuid(&self, uuid: &PlayerUuid) -> Option<Arc<Player>> {
        let players = self.players.lock();
        players.get(uuid).cloned()
    }
    pub fn get_by_id(&self, id: &PlayerId) -> Option<Arc<Player>> {
        let uuids = self.ids_to_uuids.lock();
        let uuid = uuids.get(id.0)?.as_ref()?;
        let players = self.players.lock();
        players.get(uuid).cloned()
    }
}
