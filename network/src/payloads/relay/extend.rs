// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtendPayload {
    pub address: [u8; 4],
    pub port: u16,
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl ExtendPayload {
    pub fn new(node: Node, dh_key: &[u8]) -> Self {
        let mut buffer = [0; 256];
        buffer[..dh_key.len()].copy_from_slice(&dh_key);

        Self {
            address: node.ip.octets(),
            port: u16::from_be_bytes(node.port.to_be_bytes()),
            dh_key: buffer,
        }
    }

    pub fn get_node(&self) -> Node {
        Node::new(self.address.into(), self.port)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] CreatePayload::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] CreatePayload::deserialize --> Unable to deserialize payload")
    }
}

impl Default for ExtendPayload {
    fn default() -> Self {
        let node = Node::default();
        Self {
            address: node.ip.octets(),
            port: node.port,
            dh_key: [0; 256],
        }
    }
}
