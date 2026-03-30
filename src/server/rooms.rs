use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        nonpoison::{Mutex, RwLock},
    },
};

use derive_more::Deref;

use crate::room::{Room, ids::MapId};

#[derive(Default, Deref)]
pub struct Rooms {
    pub rooms: Mutex<BTreeMap<MapId, Arc<RwLock<Room>>>>,
}
impl Rooms {
    pub fn get_by_id(
        rooms: &mut BTreeMap<MapId, Arc<RwLock<Room>>>,
        id: MapId,
    ) -> &mut Arc<RwLock<Room>> {
        rooms
            .entry(id)
            .or_insert_with(move || Arc::new(RwLock::new(Room::new(id))))
    }
}
