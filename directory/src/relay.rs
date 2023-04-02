use crate::*;
use openssl::rsa::Rsa;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Relay {
    pub nickname: String,
    pub identity_key: Vec<u8>,
    pub address: SocketAddr,
    pub contact_information: String,
}

impl Relay {
    pub fn new(
        nickname: String,
        identity_key: Vec<u8>,
        address: SocketAddr,
        contact_information: String,
    ) -> Self {
        Self {
            nickname,
            identity_key,
            address,
            contact_information,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Relay::serialize --> Unable to serialize")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Relay::deserialize --> Unable to deserialize")
    }
}
