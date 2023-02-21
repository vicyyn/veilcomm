// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub const CELL_SIZE: usize = 512;
pub const CELL_PAYLOAD_SIZE: usize = 509;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cell {
    pub circ_id: u16,
    pub command: u8,
    #[serde(with = "BigArray")]
    pub payload: [u8; CELL_PAYLOAD_SIZE],
}

impl Cell {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Cell {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            circ_id: 0,
            command: 0,
            payload: [0; CELL_PAYLOAD_SIZE],
        }
    }
}
