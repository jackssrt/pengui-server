use std::sync::Arc;

use rand::distr::{Alphanumeric, SampleString};
use serde::Serialize;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize)]
#[repr(transparent)]
pub struct PlayerUuid(pub String);
impl PlayerUuid {
    pub fn new_random() -> Self {
        // rand::rng() returns a ThreadRng
        // which is cryptographically secure
        // according to the rand crate docs
        Self(Alphanumeric.sample_string(&mut rand::rng(), 16))
    }
}
impl Default for PlayerUuid {
    fn default() -> Self {
        Self::new_random()
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Serialize)]
#[repr(transparent)]
pub struct PlayerId(pub usize);
