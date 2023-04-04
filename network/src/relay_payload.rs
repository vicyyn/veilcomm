use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelayPayload {
    pub command: u8,
    pub recognized: u16,
    pub stream_id: u16,
    pub digest: u32,
    pub length: u16,
    #[serde(with = "BigArray")]
    pub data: [u8; PAYLOAD_LEN - 11],
}

impl From<Payload> for RelayPayload {
    fn from(value: Payload) -> Self {
        bincode::deserialize(&value.serialize().to_vec()).unwrap()
    }
}

impl RelayPayload {
    pub fn new(data: [u8; PAYLOAD_LEN]) -> Self {
        Self {
            command: data[0],
            recognized: u16::from_le_bytes(data[1..3].try_into().unwrap()),
            stream_id: u16::from_le_bytes(data[3..5].try_into().unwrap()),
            digest: u32::from_le_bytes(data[5..9].try_into().unwrap()),
            length: u16::from_le_bytes(data[9..11].try_into().unwrap()),
            data: data[11..].try_into().unwrap(),
        }
    }

    pub fn into_extend(&self) -> ExtendPayload {
        ExtendPayload {
            address: self.data[..4].try_into().unwrap(),
            port: u16::from_le_bytes(self.data[4..6].try_into().unwrap()),
            onion_skin: OnionSkin::deserialize(&self.data[6..(6 + ONION_SKIN_LEN)]),
        }
    }

    pub fn into_extended(&self) -> ExtendedPayload {
        let mut dh_key = [0; 256];
        dh_key.copy_from_slice(&self.data[0..256]);
        ExtendedPayload { dh_key }
    }

    pub fn new_extend_payload(extend_payload: ExtendPayload) -> Self {
        let data = extend_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 6,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_extended_payload(extended_payload: ExtendedPayload) -> Self {
        let data = extended_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 7,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_data_payload(data: &[u8]) -> Self {
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 2,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }
}

impl Default for RelayPayload {
    fn default() -> Self {
        Self {
            command: 0,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: [0; PAYLOAD_LEN - 11],
        }
    }
}
