/// Privacy settings, stored on the client and restored on every connection using session commands, why?
#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PrivacySettings {
    pub private: bool,
    pub single_player: bool,
    pub should_hide_location: bool,
    pub hide_unnamed_players: bool,
}

impl PrivacySettings {
    /// construct a new `PrivacySettings` with the most private settings of two `PrivacySettings`
    pub const fn or(&self, other: &Self) -> Self {
        Self {
            private: self.private || other.private,
            single_player: self.single_player || other.single_player,
            hide_unnamed_players: self.hide_unnamed_players || other.hide_unnamed_players,
            should_hide_location: self.should_hide_location || other.should_hide_location,
        }
    }
}
