use std::{
    collections::HashMap,
    sync::{
        Arc,
        nonpoison::{Mutex, RwLock},
    },
};

use crate::player::{
    Player,
    ids::{PlayerId, PlayerUuid},
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
            .unwrap_or(PlayerId(ids_to_uuids.len()))
    }
    pub fn insert_new(
        players: &mut HashMap<PlayerUuid, Arc<RwLock<Player>>>,
        ids_to_uuids: &mut Vec<Option<PlayerUuid>>,
        player: Player,
    ) -> Arc<RwLock<Player>> {
        let uuid = player.uuid.clone();
        let id = player.id.0;
        let wrapped_player = Arc::new(RwLock::new(player));
        players.insert(uuid.clone(), wrapped_player.clone());
        if let Some(x) = ids_to_uuids.get_mut(id) {
            *x = Some(uuid);
        } else {
            // vec is too short
            ids_to_uuids.extend_one(Some(uuid));
        }
        wrapped_player
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
}
