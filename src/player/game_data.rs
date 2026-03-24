use serde::Serialize;

#[derive(Default, Serialize)]
pub struct GameData {
    pub sprite: String,
    pub sprite_index: Option<u32>,
    pub system: Option<String>,
}
