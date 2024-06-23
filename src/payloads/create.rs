use openssl::{
    pkey::Public,
    rsa::{Padding, Rsa},
    symm::{encrypt, Cipher},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnionSkin {
    pub rsa_encrypted_aes_key: Vec<u8>,
    pub aes_encrypted_dh_key: Vec<u8>,
}

impl OnionSkin {
    pub fn new(rsa: Rsa<Public>, aes: [u8; 16], dh_key: [u8; 256]) -> Self {
        let mut rsa_encrypted_aes_key: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.public_encrypt(&aes, &mut rsa_encrypted_aes_key, Padding::PKCS1)
            .unwrap();

        let aes_encrypted_dh_key = encrypt(Cipher::aes_128_ctr(), &aes, None, &dh_key).unwrap();

        Self {
            rsa_encrypted_aes_key,
            aes_encrypted_dh_key,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePayload {
    pub circuit_id: Uuid,
    pub onion_skin: OnionSkin,
}
