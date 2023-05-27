use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatedPayload {
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl CreatedPayload {
    pub fn new(dh_key: &[u8]) -> Self {
        let mut buffer = [0; 256];
        buffer[..dh_key.len()].copy_from_slice(&dh_key);
        Self { dh_key: buffer }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec()).unwrap()
    }
}

impl Default for CreatedPayload {
    fn default() -> Self {
        Self { dh_key: [0; 256] }
    }
}
