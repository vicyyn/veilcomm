// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtendedPayload {
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl From<Payload> for ExtendedPayload {
    fn from(value: Payload) -> Self {
        Self::deserialize(value.get_buffer())
    }
}

impl ExtendedPayload {
    pub fn new(dh_key: [u8; 256]) -> Self {
        ExtendedPayload { dh_key }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] CreatePayload::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] CreatePayload::deserialize --> Unable to deserialize payload")
    }
}

impl Default for ExtendedPayload {
    fn default() -> Self {
        Self { dh_key: [0; 256] }
    }
}
