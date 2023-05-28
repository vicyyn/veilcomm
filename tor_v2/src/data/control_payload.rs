use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ControlPayload {
    #[serde(with = "BigArray")]
    pub data: [u8; PAYLOAD_LEN],
}

impl From<Payload> for ControlPayload {
    fn from(value: Payload) -> Self {
        bincode::deserialize(&value.serialize().to_vec()).unwrap()
    }
}

impl ControlPayload {
    pub fn new(data: [u8; PAYLOAD_LEN]) -> Self {
        Self { data }
    }

    pub fn into_create(&self) -> CreatePayload {
        CreatePayload {
            onion_skin: OnionSkin::deserialize(&self.data[..ONION_SKIN_LEN]),
        }
    }

    pub fn into_created(&self) -> CreatedPayload {
        let mut buffer = [0; 256];
        buffer.copy_from_slice(&self.data[0..256]);
        CreatedPayload { dh_key: buffer }
    }

    pub fn new_create_payload(create_payload: CreatePayload) -> Self {
        let data = create_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN];
        buffer[..data.len()].copy_from_slice(&data);
        Self { data: buffer }
    }

    pub fn new_created_payload(created_payload: CreatedPayload) -> Self {
        let data = created_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN];
        buffer[..data.len()].copy_from_slice(&data);
        Self { data: buffer }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }
}
