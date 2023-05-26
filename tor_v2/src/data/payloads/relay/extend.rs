use crate::*;
use std::net::SocketAddrV4;

#[derive(Debug)]
pub struct ExtendPayload {
    pub ip: [u8; 4],
    pub port: u16,
    pub onion_skin: OnionSkin,
}

impl ExtendPayload {
    pub fn new(socket_address: SocketAddrV4, onion_skin: OnionSkin) -> Self {
        Self {
            ip: socket_address.ip().octets(),
            port: socket_address.port(),
            onion_skin,
        }
    }

    pub fn get_address(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.ip.into(), self.port)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.extend(self.ip);
        serialized.extend(self.port.to_le_bytes());
        serialized.extend(self.onion_skin.serialize());
        return serialized;
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        Self {
            ip: buffer[0..4].try_into().unwrap(),
            port: u16::from_le_bytes(buffer[4..6].try_into().unwrap()),
            onion_skin: OnionSkin::deserialize(&buffer[6..]),
        }
    }
}
