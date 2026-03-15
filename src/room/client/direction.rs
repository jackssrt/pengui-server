use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::FromRepr;

#[derive(
    Default,
    FromRepr,
    Serialize_repr,
    Deserialize_repr,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[repr(u8)]
pub enum Direction {
    Up,
    Right,
    #[default]
    Down,
    Left,
}
