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

impl From<ControlPayload> for CreatePayload {
    fn from(value: ControlPayload) -> Self {
        let mut buffer = [0; 256];
        buffer[..value.data.len()].copy_from_slice(&value.data);
        Self { dh_key: buffer }
    }
}

impl From<ExtendPayload> for CreatePayload {
    fn from(value: ExtendPayload) -> Self {
        Self {
            dh_key: value.dh_key,
        }
    }
}

impl CreatePayload {
    pub fn new(dh_key: &[u8]) -> Self {
        let mut buffer = [0; 256];
        buffer[..dh_key.len()].copy_from_slice(&dh_key);
        Self { dh_key: buffer }
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

impl Default for CreatePayload {
    fn default() -> Self {
        Self { dh_key: [0; 256] }
    }
}
