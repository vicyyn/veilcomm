// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatedPayload {
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl From<Payload> for CreatedPayload {
    fn from(value: Payload) -> Self {
        Self::deserialize(value.get_buffer())
    }
}

impl CreatedPayload {
    pub fn new(dh_key: [u8; 256]) -> CreatedPayload {
        CreatedPayload { dh_key }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> CreatedPayload {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }
}

impl Default for CreatedPayload {
    fn default() -> Self {
        Self { dh_key: [0; 256] }
    }
}
