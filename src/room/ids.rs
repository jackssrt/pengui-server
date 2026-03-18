use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};

/// aka room id
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug, Copy)]
#[repr(transparent)]
pub struct MapId(pub NonZeroU16);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug, Copy)]
#[repr(transparent)]
pub struct SwitchId(pub u16);
impl SwitchId {
    pub const DEBUG_MODE_2KKI: Self = Self(11);
    pub const TIME_TRIAL_2KKI: Self = Self(1430);
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug, Copy)]
#[repr(transparent)]
pub struct VariableId(pub u16);
impl VariableId {
    pub const TIME_TRIAL_ELAPSED_2KKI: Self = Self(88);
}
