use log::info;
use openssl::{
    bn::BigNum,
    dh::Dh,
    pkey::{Private, Public},
    rsa::{Padding, Rsa},
    symm::{decrypt, encrypt, Cipher},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
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

pub fn get_handshake_from_onion_skin(
    onion_skin: OnionSkin,
    dh_private: &Dh<Private>,
    rsa_private: &Rsa<Private>,
) -> Vec<u8> {
    let mut aes = vec![0; rsa_private.size() as usize];
    rsa_private
        .private_decrypt(&onion_skin.rsa_encrypted_aes_key, &mut aes, Padding::PKCS1)
        .unwrap();

    let dh = decrypt(
        Cipher::aes_128_ctr(),
        &aes[0..16],
        None,
        &onion_skin.aes_encrypted_dh_key,
    )
    .unwrap();

    let handshake = dh_private
        .compute_key(&BigNum::from_slice(&dh).unwrap())
        .unwrap();

    info!("Handshake Successful: {}", hex::encode(&handshake[0..32]));
    handshake
}
