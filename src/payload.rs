// Payload that gets sent in the cell
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload(#[serde(with = "BigArray")] [u8; PAYLOAD_SIZE]);

impl Payload {
    pub fn new(data: &[u8]) -> Self {
        let mut buffer = [0; PAYLOAD_SIZE];
        buffer[..data.len()].copy_from_slice(data);
        Self(buffer)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Cell {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self([0; PAYLOAD_SIZE])
    }
}
