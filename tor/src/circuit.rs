use crate::*;
use network::Payload;
use openssl::symm::{decrypt, encrypt, Cipher};

pub struct Circuit {
    pub id: u8,
    pub aes_key: Vec<AESKey>,
}

impl Circuit {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            aes_key: vec![],
        }
    }

    pub fn add_aes_key(&mut self, aes_key: AESKey) {
        self.aes_key.push(aes_key);
    }

    pub fn encrypt_payload(&self, payload: Payload) -> Payload {
        let encrypted_payload = encrypt(
            Cipher::aes_128_ctr(),
            &self.aes_key[0].get_key(),
            None,
            &payload.get_buffer()[..],
        )
        .unwrap();
        Payload::new(&encrypted_payload)
    }

    pub fn decrypt_payload(&self, encrypted_payload: Payload) -> Payload {
        let payload = decrypt(
            Cipher::aes_128_ctr(),
            &self.aes_key[0].get_key(),
            None,
            &encrypted_payload.get_buffer()[..],
        )
        .unwrap();
        Payload::new(&payload)
    }
}
