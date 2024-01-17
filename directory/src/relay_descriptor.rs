use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelayDescriptor {
    pub nickname: String,
    pub identity_key: Vec<u8>,
    pub socket_address: SocketAddrV4,
    pub contact_information: String,
}

impl Default for RelayDescriptor {
    fn default() -> Self {
        Self::new(
            "john".to_string(),
            vec![],
            SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8000),
            "john".to_string(),
        )
    }
}

impl RelayDescriptor {
    pub fn new(
        nickname: String,
        identity_key: Vec<u8>,
        socket_address: SocketAddrV4,
        contact_information: String,
    ) -> Self {
        Self {
            nickname,
            identity_key,
            socket_address,
            contact_information,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer).unwrap()
    }
}
