// Payload that gets sent in the cell
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub const CELL_PAYLOAD_SIZE: usize = 509;

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    pub relay_command: u8,
    pub recognized: u16,
    pub stream_id: u16,
    pub digest: u32,
    pub length: u16,
    #[serde(with = "BigArray")]
    pub data: [u8; 498],
}

impl Payload {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Cell {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.data[..data.len()].copy_from_slice(&data);
        self.length = data.len().try_into().unwrap();
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self {
            relay_command: 0,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: [0; 498],
        }
    }
}
