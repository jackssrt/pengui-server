use crate::player::ids::PlayerUuid;
pub mod ids;

pub struct Party {
    id: String,
    name: String,
    members: Option<PlayerUuid>,
}
