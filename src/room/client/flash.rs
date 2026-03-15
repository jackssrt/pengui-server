use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Flash {
    r: u8,
    g: u8,
    b: u8,
    power: u8,
    frames: u8,
}
