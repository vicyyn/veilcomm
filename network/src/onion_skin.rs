use openssl::{
    pkey::Private,
    rsa::{Padding, Rsa},
};

use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnionSkin {
    #[serde(with = "BigArray")]
    pub rsa_encrypted_aes_key: [u8; 128],
    #[serde(with = "BigArray")]
    pub aes_encrypted_dh_key: [u8; 256],
}

impl OnionSkin {
    pub fn new(data: &[u8]) -> Self {
        Self {
            rsa_encrypted_aes_key: data[0..128].try_into().unwrap(),
            aes_encrypted_dh_key: data[128..384].try_into().unwrap(),
        }
    }

    pub fn get_dh(&self, rsa: Rsa<Private>) -> [u8; 256] {
        let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.private_decrypt(&self.rsa_encrypted_aes_key, &mut buf, Padding::PKCS1_OAEP)
            .unwrap();

        let dh = decrypt(
            Cipher::aes_128_ctr(),
            &buf[0..16],
            None,
            &self.aes_encrypted_dh_key[..],
        )
        .unwrap();

        return dh.try_into().unwrap();
    }
}
