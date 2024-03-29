use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatedPayload {
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl From<ControlPayload> for CreatedPayload {
    fn from(value: ControlPayload) -> Self {
        let mut buffer = [0; 256];
        buffer[..value.data.len()].copy_from_slice(&value.data);
        Self { dh_key: buffer }
    }
}

impl CreatedPayload {
    pub fn new(dh_key: &[u8]) -> Self {
        let mut buffer = [0; 256];
        buffer[..dh_key.len()].copy_from_slice(&dh_key);
        Self { dh_key: buffer }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }
}

impl Default for CreatedPayload {
    fn default() -> Self {
        Self { dh_key: [0; 256] }
    }
}
