use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug, Deserialize)]
#[repr(transparent)]
pub struct PartyUuid(pub String);
