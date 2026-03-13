use std::{
    collections::BTreeMap,
    ops::Deref,
    sync::{
        Arc,
        nonpoison::{Mutex, RwLock},
    },
};

use crate::room::{Room, ids::MapId};

#[derive(Default)]
pub struct Rooms {
    pub rooms: Mutex<BTreeMap<MapId, Arc<RwLock<Room>>>>,
}
impl Rooms {
    pub fn get_by_id(
        rooms: &mut BTreeMap<MapId, Arc<RwLock<Room>>>,
        id: MapId,
    ) -> &mut Arc<RwLock<Room>> {
        rooms
            .entry(id.clone())
            .or_insert_with(move || Arc::new(RwLock::new(Room::new(id))))
    }
}

impl Deref for Rooms {
    type Target = Mutex<BTreeMap<MapId, Arc<RwLock<Room>>>>;

    fn deref(&self) -> &Self::Target {
        &self.rooms
    }
}
