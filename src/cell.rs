// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct Cell {
    // header
    pub circ_id: u16,
    pub command: u8,
    pub payload: Payload,
}

impl Cell {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Cell::serialize --> Unable to serialize cell")
    }

    pub fn deserialize(buffer: &[u8]) -> Cell {
        bincode::deserialize(&buffer.to_vec()).unwrap()
    }

    pub fn get_create_cell(circ_id: u16, payload: Payload) -> Cell {
        Self {
            circ_id,
            command: 1,
            payload,
        }
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
