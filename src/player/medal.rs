use serde::Serialize;
use strum::FromRepr;

#[derive(FromRepr, Default)]
#[repr(u8)]
pub enum Medal {
    #[default]
    None,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

// index with Medal

#[derive(Default, Serialize)]
pub struct Medals(pub [i32; 5]);
