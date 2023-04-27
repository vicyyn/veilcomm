use serde::{Deserialize, Serialize};

use crate::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct Introduce1Payload {
    pub address: [u8; 32],
    pub ip: [u8; 4],
    pub port: u16,
    pub cookie: [u8; 20],
}

impl Introduce1Payload {
    pub fn new(address: [u8; 32], node: Node, cookie: [u8; 20]) -> Self {
        Self {
            address,
            ip: node.ip.octets(),
            port: u16::from_be_bytes(node.port.to_be_bytes()),
            cookie,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] Introduce1::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Introduce1::deserialize --> Unable to deserialize payload")
    }
}
