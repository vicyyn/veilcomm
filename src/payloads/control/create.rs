// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePayload {
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl From<Payload> for CreatePayload {
    fn from(value: Payload) -> Self {
        Self::deserialize(value.get_buffer())
    }
}

impl CreatePayload {
    pub fn new(dh_key: [u8; 256]) -> CreatePayload {
        CreatePayload { dh_key }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] CreatePayload::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> CreatePayload {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] CreatePayload::deserialize --> Unable to deserialize payload")
    }
}

impl Default for CreatePayload {
    fn default() -> Self {
        Self { dh_key: [0; 256] }
    }
}
