use serde::{Deserialize, Serialize};
use strum::FromRepr;

#[derive(
    Serialize, Deserialize, Default, FromRepr, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(u8)]
pub enum Direction {
    #[serde(rename = "0")]
    Up,
    #[serde(rename = "1")]
    Right,
    #[serde(rename = "2")]
    #[default]
    Down,
    #[serde(rename = "3")]
    Left,
}
