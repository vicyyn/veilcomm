use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EstablishedRendPointPayload {}

impl EstablishedRendPointPayload {
    pub fn new() -> Self {
        Self {}
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self)
        .unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer)
        .unwrap()
    }
}
