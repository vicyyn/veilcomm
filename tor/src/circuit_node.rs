use crate::*;
use network::{Node, Payload};
use openssl::symm::{decrypt, encrypt, Cipher};

#[derive(Debug, Clone, Copy)]
pub struct CircuitNode {
    pub circ_id: u16,
    pub aes_key: Option<AESKey>,
    pub node: Node,
}

impl CircuitNode {
    pub fn new(circ_id: u16, aes_key: Option<AESKey>, node: Node) -> Self {
        Self {
            circ_id,
            aes_key,
            node,
        }
    }

    pub fn encrypt_payload(&self, payload: Payload) -> Payload {
        let encrypted_payload = encrypt(
            Cipher::aes_128_ctr(),
            &self.aes_key.unwrap().get_key(),
            None,
            &payload.get_buffer()[..],
        )
        .unwrap();
        Payload::new(&encrypted_payload)
    }

    pub fn decrypt_payload(&self, encrypted_payload: Payload) -> Payload {
        let payload = decrypt(
            Cipher::aes_128_ctr(),
            &self.aes_key.unwrap().get_key(),
            None,
            &encrypted_payload.get_buffer()[..],
        )
        .unwrap();
        Payload::new(&payload)
    }
}
