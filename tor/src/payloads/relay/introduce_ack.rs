use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IntroduceAckPayload {
    pub status: u8,
}

impl IntroduceAckPayload {
    pub fn new(status: u8) -> Self {
        Self { status }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] IntroduceAck::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer)
            .expect("[FAILED] IntroduceAck::deserialize --> Unable to deserialize payload")
    }
}
