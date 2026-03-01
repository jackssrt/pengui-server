use serde::{Deserialize, Serialize};
use strum::FromRepr;

#[derive(Deserialize, Serialize, FromRepr, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum Rank {
    #[default]
    User,
    Moderator,
    Administrator,
}
