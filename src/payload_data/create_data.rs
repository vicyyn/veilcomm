// CREATE Cell Data
use crate::*;
use serde::{Deserialize, Serialize};

pub const CELL_PAYLOAD_SIZE: usize = 509;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateData {
    h_data: u8,
}

impl CreateData {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Cell {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }
}

impl Default for CreateData {
    fn default() -> Self {
        Self { h_data: 0 }
    }
}
