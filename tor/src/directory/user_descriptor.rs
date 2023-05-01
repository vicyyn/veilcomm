use serde::{Deserialize, Serialize};

use crate::Node;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDescriptor {
    pub address: [u8; 32],
    pub publickey: Vec<u8>,
    pub introduction_points: Vec<Node>,
}

impl UserDescriptor {
    pub fn new(address: [u8; 32], publickey: Vec<u8>, introduction_points: Vec<Node>) -> Self {
        Self {
            address,
            publickey,
            introduction_points,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] UserDescriptor::serialize --> Unable to serialize")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] UserDescriptor::deserialize --> Unable to deserialize")
    }
}
