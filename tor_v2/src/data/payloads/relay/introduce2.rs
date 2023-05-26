use std::net::SocketAddrV4;

use crate::{Introduce1Payload, OnionSkin};

#[derive(Debug)]
pub struct Introduce2Payload {
    pub ip: [u8; 4],
    pub port: u16,
    pub cookie: [u8; 20],
    pub onion_skin: OnionSkin,
}

impl From<Introduce1Payload> for Introduce2Payload {
    fn from(value: Introduce1Payload) -> Self {
        Self {
            ip: value.ip,
            port: value.port,
            cookie: value.cookie,
            onion_skin: value.onion_skin,
        }
    }
}

impl Introduce2Payload {
    pub fn new(socket_address: SocketAddrV4, cookie: [u8; 20], onion_skin: OnionSkin) -> Self {
        Self {
            ip: socket_address.ip().octets(),
            port: socket_address.port(),
            cookie,
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
        serialized.extend(self.cookie);
        serialized.extend(self.onion_skin.serialize());
        return serialized;
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        Self {
            ip: buffer[0..4].try_into().unwrap(),
            port: u16::from_le_bytes(buffer[4..6].try_into().unwrap()),
            cookie: buffer[6..26].try_into().unwrap(),
            onion_skin: OnionSkin::deserialize(&buffer[26..]),
        }
    }
}
