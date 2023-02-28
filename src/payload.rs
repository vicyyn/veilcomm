// Cell Payload
use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload(#[serde(with = "BigArray")] [u8; PAYLOAD_SIZE]);

impl Payload {
    pub fn new(data: &[u8]) -> Self {
        let mut buffer = [0; PAYLOAD_SIZE];
        buffer[..data.len()].copy_from_slice(data);
        Self(buffer)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Payload {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self([0; PAYLOAD_SIZE])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload() {
        // let payload = Payload::default();
        // let create_payload: CreatePayload = payload.into();
        // println!("{:?}", create_payload);
    }
}
