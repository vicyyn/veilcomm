// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtendPayload {
    pub address: [u8; 4],
    pub port: [u8; 2],
    #[serde(with = "BigArray")]
    pub dh_key: [u8; 256],
}

impl From<Payload> for ExtendPayload {
    fn from(value: Payload) -> Self {
        Self::deserialize(value.get_buffer())
    }
}

impl Into<CreatePayload> for ExtendPayload {
    fn into(self) -> CreatePayload {
        CreatePayload {
            dh_key: self.dh_key,
        }
    }
}

impl ExtendPayload {
    pub fn new(node: Node, dh_key: [u8; 256]) -> Self {
        Self {
            address: node.ip.octets(),
            port: node.port.to_be_bytes(),
            dh_key,
        }
    }

    pub fn get_node(&self) -> Node {
        Node::new(
            self.address.into(),
            ((self.port[0] as u16) << 8) + (self.port[1] as u16),
        )
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
            port: node.port.to_be_bytes(),
            dh_key: [0; 256],
        }
    }
}
