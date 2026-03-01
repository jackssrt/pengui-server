use crate::room::minigame::Minigame;

pub mod badges;
pub mod client;
pub mod ids;
pub mod minigame;
pub struct Room {
    pub id: isize,
    pub singleplayer: bool,
    // pub clients: Vec<Arc<RoomClient>>,
    // TODO
    // pub conditions: Vec<Condition>,
    pub minigame: Option<Minigame>,
}
