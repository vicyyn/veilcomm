use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rendezvous1Payload {
    pub cookie: [u8; 20],
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl Rendezvous1Payload {
    pub fn new(cookie: [u8; 20], dh_key: [u8; 256]) -> Self {
        Self { cookie, dh_key }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] Rendezvous1::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rendezvous1::deserialize --> Unable to deserialize payload")
    }
}
