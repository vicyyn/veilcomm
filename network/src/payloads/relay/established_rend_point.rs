use serde::{Deserialize, Serialize};

use crate::EstablishRendPointPayload;

#[derive(Debug, Serialize, Deserialize)]
pub struct EstablishedRendPointPayload {
    pub cookie: [u8; 20],
}

impl From<EstablishRendPointPayload> for EstablishedRendPointPayload {
    fn from(value: EstablishRendPointPayload) -> Self {
        Self {
            cookie: value.cookie,
        }
    }
}

impl EstablishedRendPointPayload {
    pub fn new(cookie: [u8; 20]) -> Self {
        Self { cookie }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self)
            .expect("[FAILED] EstablishedRendPoint::serialize --> Unable to serialize payload")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer)
            .expect("[FAILED] EstablishedRendPoint::deserialize --> Unable to deserialize payload")
    }
}
