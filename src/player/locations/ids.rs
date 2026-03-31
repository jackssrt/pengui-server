use serde::Serialize;

#[derive(Serialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct LocationId(pub i32);
