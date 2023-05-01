use crate::*;
use openssl::symm::{decrypt, encrypt, Cipher};

#[derive(Debug, Clone)]
pub struct CircuitNode {
    pub aes_key: Option<AESKey>,
    pub node: Node,
}

impl CircuitNode {
    pub fn new(aes_key: Option<AESKey>, node: Node) -> Self {
        Self { aes_key, node }
    }

    pub fn encrypt_payload(&self, payload: Payload) -> Payload {
        match payload {
            Payload::RelayPayload(relay_payload) => {
                return Payload::new_relay_payload(RelayPayload::new(
                    self.encrypt_data(relay_payload.serialize())
                        .try_into()
                        .unwrap(),
                ));
            }
            Payload::ControlPayload(control_payload) => {
                Payload::new_control_payload(ControlPayload::new(
                    self.encrypt_data(control_payload.serialize())
                        .try_into()
                        .unwrap(),
                ))
            }
        }
    }

    pub fn decrypt_payload(&self, payload: Payload) -> Payload {
        match payload {
            Payload::RelayPayload(relay_payload) => Payload::new_relay_payload(RelayPayload::new(
                self.decrypt_data(relay_payload.serialize())
                    .try_into()
                    .unwrap(),
            )),
            Payload::ControlPayload(control_payload) => {
                Payload::new_control_payload(ControlPayload::new(
                    self.decrypt_data(control_payload.serialize())
                        .try_into()
                        .unwrap(),
                ))
            }
        }
    }

    pub fn encrypt_data(&self, data: Vec<u8>) -> Vec<u8> {
        encrypt(
            Cipher::aes_128_ctr(),
            &self.aes_key.unwrap().get_key(),
            None,
            &data[..],
        )
        .unwrap()
    }

    pub fn decrypt_data(&self, data: Vec<u8>) -> Vec<u8> {
        decrypt(
            Cipher::aes_128_ctr(),
            &self.aes_key.unwrap().get_key(),
            None,
            &data[..],
        )
        .unwrap()
    }
}
