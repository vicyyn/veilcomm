use serde::{Deserialize, Serialize};

use crate::{Introduce1Payload, Node};

#[derive(Debug, Serialize, Deserialize)]
pub struct Introduce2Payload {
    pub ip: [u8; 4],
    pub port: u16,
    pub cookie: [u8; 20],
}

impl From<Introduce1Payload> for Introduce2Payload {
    fn from(value: Introduce1Payload) -> Self {
        Self {
            ip: value.ip,
            port: value.port,
            cookie: value.cookie,
        }
    }
}

impl Introduce2Payload {
    pub fn new(node: Node, cookie: [u8; 20]) -> Self {
        Self {
            ip: node.ip.octets(),
            port: u16::from_be_bytes(node.port.to_be_bytes()),
            cookie,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] Introduce2::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Introduce2::deserialize --> Unable to deserialize payload")
    }
}
