use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Rendezvous2Payload {}

impl Rendezvous2Payload {
    pub fn new() -> Self {
        Self {}
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] Rendezvous2::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rendezvous2::deserialize --> Unable to deserialize payload")
    }
}
