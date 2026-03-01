#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PrivacySettings {
    pub private: bool,
    pub single_player: bool,
    pub should_hide_location: bool,
    pub hide_unnamed_players: bool,
}
