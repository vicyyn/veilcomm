use crate::BeginPayload;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectedPayload {
    pub ip: [u8; 4],
    pub port: u16,
}

impl From<BeginPayload> for ConnectedPayload {
    fn from(value: BeginPayload) -> Self {
        Self {
            ip: value.ip,
            port: value.port,
        }
    }
}

impl ConnectedPayload {
    pub fn new(socket_address: SocketAddrV4) -> Self {
        Self {
            ip: socket_address.ip().octets(),
            port: socket_address.port(),
        }
    }

    pub fn get_address(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.ip.into(), self.port)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec()).unwrap()
    }
}
