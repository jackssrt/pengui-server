use std::{ops::Deref, sync::Arc};

use rand::distr::{Alphabetic, SampleString};
use serde::Serialize;

use crate::traits::Random;

#[derive(Debug, Clone, Serialize)]
pub struct MessageId(pub Arc<str>);
impl Random for MessageId {
    fn random() -> Self {
        Self(Alphabetic.sample_string(&mut rand::rng(), 12).into())
    }
}
impl Deref for MessageId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
