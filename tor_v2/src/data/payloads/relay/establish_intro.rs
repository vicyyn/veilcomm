use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EstablishIntroPayload {
    pub address: [u8; 32],
}

impl EstablishIntroPayload {
    pub fn new(address: [u8; 32]) -> Self {
        Self { address }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec()).unwrap()
    }
}
