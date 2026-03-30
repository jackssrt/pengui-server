use std::sync::Arc;

use derive_more::Deref;
use rand::distr::{Alphabetic, SampleString};
use serde::Serialize;

use crate::traits::Random;

#[derive(Debug, Clone, Deref, Serialize)]
pub struct MessageId(pub Arc<str>);
impl Random for MessageId {
    fn random() -> Self {
        Self(Alphabetic.sample_string(&mut rand::rng(), 12).into())
    }
}
