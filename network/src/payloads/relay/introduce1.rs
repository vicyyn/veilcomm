use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Introduce1Payload {
    pub address: [u8; 32],
}

impl Introduce1Payload {
    pub fn new(address: [u8; 32]) -> Self {
        Self { address }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] Introduce1::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Introduce1::deserialize --> Unable to deserialize payload")
    }
}
