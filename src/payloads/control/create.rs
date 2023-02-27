// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePayload {
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
    #[serde(with = "BigArray")]
    pub padding: [u8; PAYLOAD_SIZE - 256],
}

impl CreatePayload {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Cell {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }

    pub fn set_dh_key(&mut self, dh_key: &[u8]) {
        self.dh_key[..dh_key.len()].copy_from_slice(&dh_key);
    }

    pub fn get_create_cell(dh_key: &[u8]) -> CreatePayload {
        let mut create_payload = CreatePayload::default();
        create_payload.set_dh_key(dh_key);
        create_payload
    }
}

impl Default for CreatePayload {
    fn default() -> Self {
        Self {
            dh_key: [0; 256],
            padding: [0; PAYLOAD_SIZE - 256],
        }
    }
}
