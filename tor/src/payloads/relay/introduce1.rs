use crate::{Node, OnionSkin};

#[derive(Debug)]
pub struct Introduce1Payload {
    pub address: [u8; 32],
    pub ip: [u8; 4],
    pub port: u16,
    pub cookie: [u8; 20],
    pub onion_skin: OnionSkin,
}

impl Introduce1Payload {
    pub fn new(address: [u8; 32], node: Node, cookie: [u8; 20], onion_skin: OnionSkin) -> Self {
        Self {
            address,
            ip: node.ip.octets(),
            port: u16::from_be_bytes(node.port.to_be_bytes()),
            cookie,
            onion_skin,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.extend(self.address);
        serialized.extend(self.ip);
        serialized.extend(self.port.to_le_bytes());
        serialized.extend(self.cookie);
        serialized.extend(self.onion_skin.serialize());
        serialized
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        Self {
            address: buffer[0..32].try_into().unwrap(),
            ip: buffer[32..36].try_into().unwrap(),
            port: u16::from_le_bytes(buffer[36..38].try_into().unwrap()),
            cookie: buffer[38..58].try_into().unwrap(),
            onion_skin: OnionSkin::deserialize(&buffer[58..]),
        }
    }
}
