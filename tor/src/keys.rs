use openssl::{bn::BigNum, dh::Dh, pkey::PKey, pkey::Private, rsa::Rsa};
use std::convert::From;

pub const KEY_LEN: usize = 32;

#[derive(Debug, Copy, Clone)]
pub struct AESKey([u8; 16]);

impl From<&[u8]> for AESKey {
    fn from(value: &[u8]) -> Self {
        Self(value.try_into().unwrap())
    }
}

impl AESKey {
    pub fn get_key(&self) -> [u8; 16] {
        self.0
    }
}

pub struct Keys {
    pub relay_id_rsa: Rsa<Private>,
    pub onion_tap: Rsa<Private>,
    pub conn_tls: Rsa<Private>,
    pub ntor: PKey<Private>,
    pub relay_id_ed: PKey<Private>,
    pub relay_sign_ed: PKey<Private>,
    pub link_ed: PKey<Private>,
    pub dh: Dh<Private>,
}

impl Keys {
    pub fn new() -> Self {
        let relay_id_rsa = Rsa::generate(2048).unwrap();
        let onion_tap = Rsa::generate(2048).unwrap();
        let conn_tls = Rsa::generate(2048).unwrap();

        let ntor = PKey::generate_x25519().unwrap();

        let relay_id_ed = PKey::generate_ed25519().unwrap();
        let relay_sign_ed = PKey::generate_ed25519().unwrap();
        let link_ed = PKey::generate_ed25519().unwrap();

        Self {
            relay_id_rsa,
            onion_tap,
            conn_tls,
            ntor,
            relay_id_ed,
            relay_sign_ed,
            link_ed,
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
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use openssl::{pkey::Public, rsa::Padding};
//
//     #[test]
//     fn test_rsa() {
//         let my_keys = Keys::new();
//         let rsa = my_keys.relay_id_rsa;
//         let public_key = Rsa::public_key_from_pem(&rsa.public_key_to_pem().unwrap()).unwrap();
//
//         // let data = b"foobar";
//         // println!("{}", data.len());
//         // let mut encrypted = vec![0; rsa.size() as usize];
//         // let _ = rsa
//         //     .public_encrypt(data, &mut encrypted, Padding::PKCS1)
//         //     .unwrap();
//         // println!("{} ", encrypted.len());
//         // let mut decrypted = vec![0; encrypted.len() as usize];
//         // let bytes = rsa
//         //     .private_decrypt(&encrypted, &mut decrypted, Padding::PKCS1)
//         //     .unwrap();
//         // println!("{:?}", from_utf8(&decrypted[0..bytes]).unwrap());
//
//         // let len = my_keys.onion_tap.private_key_to_pem().unwrap();
//         // println!("{}", len.len());
//     }
// }
