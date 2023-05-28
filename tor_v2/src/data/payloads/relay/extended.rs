use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtendedPayload {
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl From<CreatedPayload> for ExtendedPayload {
    fn from(value: CreatedPayload) -> Self {
        Self {
            dh_key: value.dh_key,
        }
    }
}

impl ExtendedPayload {
    pub fn new(dh_key: [u8; 256]) -> Self {
        ExtendedPayload { dh_key }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec()).unwrap()
    }
}

impl Default for ExtendedPayload {
    fn default() -> Self {
        Self { dh_key: [0; 256] }
    }
}
