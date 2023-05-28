use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EstablishRendPointPayload {
    pub cookie: [u8; 20],
}

impl EstablishRendPointPayload {
    pub fn new(cookie: [u8; 20]) -> Self {
        Self { cookie }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer)
        .unwrap()
    }
}
