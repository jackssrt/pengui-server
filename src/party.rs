use crate::{party::ids::PartyId, player::ids::PlayerUuid};
pub mod ids;

pub struct Party {
    id: PartyId,
    name: String,
    members: Option<PlayerUuid>,
}
