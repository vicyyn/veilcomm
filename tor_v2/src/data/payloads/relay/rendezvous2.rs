use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::Rendezvous1Payload;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rendezvous2Payload {
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl From<Rendezvous1Payload> for Rendezvous2Payload {
    fn from(value: Rendezvous1Payload) -> Self {
        Self {
            dh_key: value.dh_key,
        }
    }
}

impl Rendezvous2Payload {
    pub fn new(dh_key: [u8; 256]) -> Self {
        Self { dh_key }
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
