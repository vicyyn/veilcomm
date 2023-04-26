use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Rendezvous1Payload {}

impl Rendezvous1Payload {
    pub fn new() -> Self {
        Self {}
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
