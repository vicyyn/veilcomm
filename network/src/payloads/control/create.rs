// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePayload {
    pub onion_skin: OnionSkin,
}

// impl From<ControlPayload> for CreatePayload {
//     fn from(value: ControlPayload) -> Self {
//         let mut buffer = [0; 256];
//         buffer[..value.data.len()].copy_from_slice(&value.data);
//         Self { dh_key: buffer }
//     }
// }

impl From<ExtendPayload> for CreatePayload {
    fn from(value: ExtendPayload) -> Self {
        Self {
            onion_skin: value.onion_skin,
        }
    }
}

impl CreatePayload {
    pub fn new(onion_skin: OnionSkin) -> Self {
        Self { onion_skin }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.onion_skin.serialize()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        Self {
            onion_skin: OnionSkin::deserialize(buffer),
        }
    }
}
