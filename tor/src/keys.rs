use directory::UserDescriptor;
use network::Node;
use openssl::{bn::BigNum, dh::Dh, pkey::Private, rand::rand_bytes, rsa::Rsa};

use crate::AESKey;

pub fn generate_random_aes_key() -> [u8; 16] {
    let mut key = [0u8; 16];
    rand_bytes(&mut key).unwrap();
    key
}

pub fn generate_random_address() -> [u8; 32] {
    let mut address = [0u8; 32];
    rand_bytes(&mut address).unwrap();
    address
}

pub struct Keys {
    pub relay_id_rsa: Rsa<Private>,
    pub address: [u8; 32],
    pub user_private: Rsa<Private>,
    pub dh: Dh<Private>,
}

impl Keys {
    pub fn new() -> Self {
        let relay_id_rsa = Rsa::generate(1024).unwrap();
        let address = generate_random_address();
        let user_private = Rsa::generate(1024).unwrap();

        Self {
            relay_id_rsa,
            address,
            user_private,
            dh: Dh::get_2048_256().unwrap().generate_key().unwrap(),
        }
    }

    pub fn compute_dh(&self, half_dh: &[u8]) -> Vec<u8> {
        self.dh
            .compute_key(&BigNum::from_slice(half_dh).unwrap())
            .unwrap()
    }

    pub fn compute_aes_key(&self, half_dh: &[u8]) -> AESKey {
        let dh = self.compute_dh(half_dh);
        dh[0..16].try_into().unwrap()
    }

    pub fn get_user_descriptor(&self, introduction_points: Vec<Node>) -> UserDescriptor {
        UserDescriptor::new(
            self.address,
            self.user_private.public_key_to_der().unwrap(),
            introduction_points,
        )
    }
}
