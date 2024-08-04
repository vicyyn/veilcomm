use anyhow::Result;
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
    pub fn new(rsa: Rsa<Public>, aes: [u8; 16], dh_key: [u8; 256]) -> Result<Self> {
        let mut rsa_encrypted_aes_key: Vec<u8> = vec![0; rsa.size() as usize];
        rsa.public_encrypt(&aes, &mut rsa_encrypted_aes_key, Padding::PKCS1)?;
        let aes_encrypted_dh_key = encrypt(Cipher::aes_128_ctr(), &aes, None, &dh_key)?;
        Ok(Self {
            rsa_encrypted_aes_key,
            aes_encrypted_dh_key,
        })
    }
}

pub fn get_handshake_from_onion_skin(
    onion_skin: OnionSkin,
    dh_private: &Dh<Private>,
    rsa_private: &Rsa<Private>,
) -> Result<Vec<u8>> {
    let mut aes = vec![0; rsa_private.size() as usize];
    rsa_private.private_decrypt(&onion_skin.rsa_encrypted_aes_key, &mut aes, Padding::PKCS1)?;

    let dh = decrypt(
        Cipher::aes_128_ctr(),
        &aes[0..16],
        None,
        &onion_skin.aes_encrypted_dh_key,
    )?;

    let public_key = &BigNum::from_slice(&dh)?;
    let handshake = dh_private.compute_key(public_key)?;
    Ok(handshake)
}

#[cfg(test)]
mod tests {
    use crate::generate_random_aes_key;

    use super::*;
    use openssl::dh::Dh;
    use openssl::rsa::Rsa;

    #[test]
    fn test_onion_skin_new() {
        let rsa = Rsa::generate(2048).unwrap();
        let aes = generate_random_aes_key();
        let dh_key = [42u8; 256];
        let onion_skin = OnionSkin::new(
            Rsa::public_key_from_pem(&rsa.public_key_to_pem().unwrap()).unwrap(),
            aes,
            dh_key,
        )
        .unwrap();

        assert_eq!(onion_skin.rsa_encrypted_aes_key.len(), rsa.size() as usize);
        assert_eq!(onion_skin.aes_encrypted_dh_key.len(), 256);
    }

    #[test]
    fn test_onion_skin_serialization() {
        let rsa = Rsa::generate(2048).unwrap();
        let aes = generate_random_aes_key();
        let dh_key = [42u8; 256];

        let onion_skin = OnionSkin::new(
            Rsa::public_key_from_pem(&rsa.public_key_to_pem().unwrap()).unwrap(),
            aes,
            dh_key,
        )
        .unwrap();

        let serialized = serde_json::to_string(&onion_skin).unwrap();
        let deserialized: OnionSkin = serde_json::from_str(&serialized).unwrap();

        assert_eq!(onion_skin, deserialized);
    }

    #[test]
    fn test_handshake_between_two_users() {
        let rsa_bob = Rsa::generate(2048).unwrap();
        let dh_params = Dh::get_2048_256().unwrap();
        let dh_alice = dh_params.generate_key().unwrap();
        let dh_params = Dh::get_2048_256().unwrap();
        let dh_bob = dh_params.generate_key().unwrap();

        let aes_key = generate_random_aes_key();
        let bob_public_key =
            Rsa::public_key_from_pem(&rsa_bob.public_key_to_pem().unwrap()).unwrap();
        let onion_skin = OnionSkin::new(
            bob_public_key,
            aes_key,
            dh_alice.public_key().to_vec().try_into().unwrap(),
        )
        .unwrap();

        let bob_handshake = get_handshake_from_onion_skin(onion_skin, &dh_bob, &rsa_bob).unwrap();
        let alice_handshake = dh_alice
            .compute_key(&BigNum::from_slice(&dh_bob.public_key().to_vec()).unwrap())
            .unwrap();

        assert_eq!(alice_handshake, bob_handshake);
    }
}
