use sha1::{Digest, Sha1};
use tracing::instrument;

const PSK: &[u8] = include_bytes!("../../../key.bin");
#[derive(Debug)]
pub struct Cryptography {
    pub key: u32,
    pub counter: u32,
}

impl Cryptography {
    pub fn new() -> Self {
        Self {
            key: rand::random(),
            counter: 0,
        }
    }
    #[instrument]
    fn verify_signature(&self, data: &[u8]) -> bool {
        let mut sha = Sha1::new();
        sha.update(PSK);
        sha.update(self.key.to_be_bytes());
        sha.update(&data[4..]);
        let hash = sha.finalize();
        (AsRef::<[u8]>::as_ref(&hash))[..4] == data[..4]
    }
    const fn verify_counter(&mut self, data: &[u8]) -> bool {
        let client_counter = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        // ... the client uses uninitialized memory as the starting value of the counter
        // so we have to just accept any number higher than the counter we already have
        let result = client_counter > self.counter;
        if result {
            self.counter = client_counter;
        }
        result
    }
    pub fn verify_bytes<'a>(&mut self, data: &'a [u8]) -> Option<&'a [u8]> {
        (self.verify_signature(data) && self.verify_counter(data)).then_some(&data[8..])
    }
}
