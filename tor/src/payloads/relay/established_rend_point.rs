use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EstablishedRendPointPayload {}

impl Default for EstablishedRendPointPayload {
    fn default() -> Self {
        Self::new()
    }
}

impl EstablishedRendPointPayload {
    pub fn new() -> Self {
        Self {}
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self)
            .expect("[FAILED] EstablishedRendPoint::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer)
            .expect("[FAILED] EstablishedRendPoint::deserialize --> Unable to deserialize payload")
    }
}
