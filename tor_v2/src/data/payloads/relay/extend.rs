use crate::*;

#[derive(Debug)]
pub struct ExtendPayload {
    pub address: [u8; 4],
    pub port: u16,
    pub onion_skin: OnionSkin,
}

impl ExtendPayload {
    pub fn new(node: SocketAddrV4, onion_skin: OnionSkin) -> Self {
        Self {
            address: node.ip().octets(),
            port: u16::from_be_bytes(node.port().to_be_bytes()),
            onion_skin,
        }
    }

    pub fn get_address(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.address.into(), self.port)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.extend(self.address);
        serialized.extend(self.port.to_le_bytes());
        serialized.extend(self.onion_skin.serialize());
        return serialized;
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        Self {
            address: buffer[0..4].try_into().unwrap(),
            port: u16::from_le_bytes(buffer[4..6].try_into().unwrap()),
            onion_skin: OnionSkin::deserialize(&buffer[6..]),
        }
    }
}
