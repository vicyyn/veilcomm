use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserDescriptor {
    pub address: [u8; 32],
    pub publickey: Vec<u8>,
    pub introduction_points: Vec<SocketAddrV4>,
}

impl UserDescriptor {
    pub fn new(
        address: [u8; 32],
        publickey: Vec<u8>,
        introduction_points: Vec<SocketAddrV4>,
    ) -> Self {
        Self {
            address,
            publickey,
            introduction_points,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec()).unwrap()
    }
}
