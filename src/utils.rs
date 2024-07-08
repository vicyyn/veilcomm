use ::rand::{thread_rng, Rng};
use log::info;
use openssl::{
    bn::BigNum,
    dh::Dh,
    pkey::Private,
    rsa::{Padding, Rsa},
    symm::{decrypt, Cipher},
};

use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

use crate::OnionSkin;

pub type Connections = Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<OwnedWriteHalf>>>>>;

pub fn generate_random_aes_key() -> [u8; 16] {
    let mut rand = thread_rng();
    let key = rand.gen::<[u8; 16]>();
    key
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
    return handshake;
}
