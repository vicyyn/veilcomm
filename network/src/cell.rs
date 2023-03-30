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
    pub fn new(circ_id: u16, command: u8, payload: Payload) -> Self {
        Self {
            circ_id,
            command,
            payload,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Cell::serialize --> Unable to serialize cell")
    }

    pub fn deserialize(buffer: &[u8]) -> Cell {
        bincode::deserialize(&buffer.to_vec()).unwrap()
    }

    pub fn new_create_cell(circ_id: u16, create_payload: CreatePayload) -> Cell {
        Cell::new(circ_id, 1, create_payload.into())
    }

    pub fn new_created_cell(circ_id: u16, created_payload: CreatedPayload) -> Cell {
        Cell::new(circ_id, 2, created_payload.into())
    }

    pub fn new_ping_cell() -> Cell {
        Cell::new(0, 13, Payload::default())
    }

    pub fn new_pong_cell() -> Cell {
        Cell::new(0, 14, Payload::default())
    }

    pub fn new_extend_cell(circ_id: u16, extend_payload: ExtendPayload) -> Cell {
        Cell::new(circ_id, 15, extend_payload.into())
    }

    pub fn new_extended_cell(circ_id: u16, extended_payload: ExtendedPayload) -> Cell {
        Cell::new(circ_id, 16, extended_payload.into())
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
