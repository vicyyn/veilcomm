use std::net::SocketAddrV4;

use serde::{Deserialize, Serialize};

use crate::BeginPayload;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectedPayload {
    pub address: [u8; 4],
    pub port: u16,
}

impl From<BeginPayload> for ConnectedPayload {
    fn from(value: BeginPayload) -> Self {
        Self {
            address: value.address,
            port: value.port,
        }
    }
}

impl ConnectedPayload {
    pub fn new(node: SocketAddrV4) -> Self {
        Self {
            address: node.ip().octets(),
            port: u16::from_be_bytes(node.port().to_be_bytes()),
        }
    }

    pub fn get_address(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.address.into(), self.port)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] ConnectedPayload::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] ConnectedPayload::deserialize --> Unable to deserialize payload")
    }
}
