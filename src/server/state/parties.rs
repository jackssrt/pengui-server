use std::{
    collections::HashMap,
    sync::{Arc, nonpoison::Mutex},
};

use crate::party::{Party, ids::PartyId};

#[derive(Default)]
pub struct Parties {
    pub parties: Mutex<Arc<HashMap<PartyId, Party>>>,
}
