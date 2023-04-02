use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Relay {
    pub nickname: String,
    pub identity_key: String,
    pub address: SocketAddr,
    pub contact_information: String,
}

impl Relay {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Relay::serialize --> Unable to serialize")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Relay::deserialize --> Unable to deserialize")
    }

    pub fn default() -> Self {
        Self {
            nickname: "John".to_string(),
            identity_key: "IDENTITY_KEY".to_string(),
            address: new_socket_addr(8090),
            contact_information: "john@gmail.com".to_string(),
        }
    }
}
