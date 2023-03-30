use crate::*;
// use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub const PAYLOAD_SIZE: usize = 509;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Payload(#[serde(with = "BigArray")] [u8; PAYLOAD_SIZE]);

impl From<ExtendedPayload> for Payload {
    fn from(value: ExtendedPayload) -> Self {
        Payload::new(&value.serialize())
    }
}

impl From<ExtendPayload> for Payload {
    fn from(value: ExtendPayload) -> Self {
        Payload::new(&value.serialize())
    }
}

impl From<CreatePayload> for Payload {
    fn from(value: CreatePayload) -> Self {
        Payload::new(&value.serialize())
    }
}

impl From<CreatedPayload> for Payload {
    fn from(value: CreatedPayload) -> Self {
        Payload::new(&value.serialize())
    }
}

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

    // pub fn encrypt(&self, aes_key: AESKey) -> Payload {
    //     let encrypted_payload = encrypt(
    //         Cipher::aes_128_ctr(),
    //         &aes_key.get_key(),
    //         None,
    //         &self.get_buffer()[..],
    //     )
    //     .unwrap();
    //     Payload::new(&encrypted_payload)
    // }

    // pub fn decrypt(&self, aes_key: AESKey) -> Payload {
    //     let payload = decrypt(
    //         Cipher::aes_128_ctr(),
    //         &aes_key.get_key(),
    //         None,
    //         &self.get_buffer()[..],
    //     )
    //     .unwrap();
    //     Payload::new(&payload)
    // }

    pub fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self([0; PAYLOAD_SIZE])
    }
}
