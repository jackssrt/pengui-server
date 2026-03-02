#[derive(Default, strum::EnumIs)]
pub enum ModerationStatus {
    #[default]
    Clear,
    Muted,
    Banned,
}
impl ModerationStatus {
    pub const fn from_booleans(is_banned: bool, is_muted: bool) -> Self {
        match (is_banned, is_muted) {
            (true, ..) => Self::Banned,
            (.., true) => Self::Muted,
            _ => Self::Clear,
        }
    }
    pub const fn from_ints(banned: i8, muted: i8) -> Self {
        Self::from_booleans(banned != 0, muted != 0)
    }
}
