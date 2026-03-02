use serde::Serialize;
use strum::FromRepr;

#[derive(FromRepr)]
#[repr(u8)]
pub enum Medal {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

// index with Medal
#[derive(Default, Serialize)]
pub struct Medals(pub [i8; 5]);
