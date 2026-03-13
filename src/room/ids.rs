use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};

/// aka room id
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct MapId(pub NonZeroU16);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct SwitchId(pub u16);
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct VariableId(pub u16);
