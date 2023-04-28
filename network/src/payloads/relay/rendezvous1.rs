use crate::OnionSkin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Rendezvous1Payload {
    pub cookie: [u8; 20],
    pub onion_skin: OnionSkin,
}

impl Rendezvous1Payload {
    pub fn new(cookie: [u8; 20], onion_skin: OnionSkin) -> Self {
        Self { cookie, onion_skin }
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
