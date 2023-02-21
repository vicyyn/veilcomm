// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};

pub const CELL_SIZE: usize = 512;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cell {
    pub circ_id: u16,
    pub command: u8,
    pub payload: Payload,
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
            payload: Payload::default(),
        }
    }
}
