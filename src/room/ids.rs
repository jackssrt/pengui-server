use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct MapId(pub i16);
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct SwitchId(pub i16);
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub struct VariableId(pub i16);
