use sha1::{Digest, Sha1};

const PSK: &[u8] = include_bytes!("../../../key.bin");
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
    fn verify_signature(&self, data: &[u8]) -> bool {
        let mut sha = Sha1::new();
        sha.update(PSK);
        sha.update(self.key.to_be_bytes());
        sha.update(&data[4..]);
        let hash = sha.finalize();
        (AsRef::<[u8]>::as_ref(&hash))[..4] == data[..4]
    }
    fn verify_counter(&self, data: &[u8]) -> bool {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]]) == self.counter
    }
    pub fn verify_bytes<'a>(&self, data: &'a [u8]) -> Option<&'a [u8]> {
        (self.verify_signature(data) && self.verify_counter(data)).then_some(&data[4..])
    }
}
