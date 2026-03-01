use serde::Serialize;

#[derive(Default, Serialize)]
pub struct GameData {
    pub sprite: String,
    pub sprite_index: isize,
    pub system: String,
}
