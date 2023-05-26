use openssl::{
    pkey::{Private, Public},
    rsa::{Padding, Rsa},
};

use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub const ONION_SKIN_LEN: usize = 384;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnionSkin {
    #[serde(with = "BigArray")]
    pub rsa_encrypted_aes_key: [u8; 128],
    #[serde(with = "BigArray")]
    pub aes_encrypted_dh_key: [u8; 256],
}

impl OnionSkin {
    pub fn new(rsa: Rsa<Public>, aes: [u8; 16], dh_key: [u8; 256]) -> Self {
        let mut rsa_encrypted_aes_key: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.public_encrypt(&aes, &mut rsa_encrypted_aes_key, Padding::PKCS1)
            .unwrap();

        let aes_encrypted_dh_key = encrypt(Cipher::aes_128_ctr(), &aes, None, &dh_key).unwrap();

        Self {
            rsa_encrypted_aes_key: rsa_encrypted_aes_key.try_into().unwrap(),
            aes_encrypted_dh_key: aes_encrypted_dh_key.try_into().unwrap(),
        }
    }

    pub fn get_dh(&self, rsa: Rsa<Private>) -> [u8; 256] {
        let mut aes: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.private_decrypt(&self.rsa_encrypted_aes_key, &mut aes, Padding::PKCS1)
            .unwrap();

        let dh = decrypt(
            Cipher::aes_128_ctr(),
            &aes[0..16],
            None,
            &self.aes_encrypted_dh_key,
        )
        .unwrap();

        return dh.try_into().unwrap();
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use openssl::{bn::BigNum, dh::Dh};

    use super::*;

    #[test]
    fn test_onion_skin() {
        let rsa = Rsa::generate(1024).unwrap();
        let first_half = Dh::get_2048_256().unwrap().generate_key().unwrap();
        let second_half = Dh::get_2048_256().unwrap().generate_key().unwrap();

        let dh = Dh::get_2048_256().unwrap().generate_key().unwrap();

        let aes: [u8; 16] = first_half
            .compute_key(&BigNum::from_slice(&second_half.public_key().to_vec()).unwrap())
            .unwrap()[0..16]
            .try_into()
            .unwrap();

        let mut rsa_encrypted_aes_key: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.public_encrypt(&aes, &mut rsa_encrypted_aes_key, Padding::PKCS1)
            .unwrap();

        let aes_encrypted_dh_key =
            encrypt(Cipher::aes_128_ctr(), &aes, None, &dh.public_key().to_vec()).unwrap();

        let mut aes_decrypted: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.private_decrypt(&rsa_encrypted_aes_key, &mut aes_decrypted, Padding::PKCS1)
            .unwrap();

        let dh_key_decrypted = decrypt(
            Cipher::aes_128_ctr(),
            &aes_decrypted[0..16],
            None,
            &aes_encrypted_dh_key,
        )
        .unwrap();

        assert!(aes_decrypted[0..16].eq(&aes.to_vec()));
        assert!(dh.public_key().to_vec().eq(&dh_key_decrypted));
    }
}
