use super::*;
use anyhow::Result;
use log::info;
use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Payload {
    EstablishIntro(EstablishIntroPayload),
    Create(CreatePayload),
    Created(CreatedPayload),
    Extend(ExtendPayload),
    Extended(ExtendedPayload),
}

#[derive(PartialEq, Eq, Debug)]
pub enum PayloadType {
    Create,
    Created,
    Extend,
    Extended,
    EstablishIntro,
}

impl Payload {
    pub fn get_type(&self) -> PayloadType {
        match self {
            Payload::Create(_) => PayloadType::Create,
            Payload::Created(_) => PayloadType::Created,
            Payload::Extend(_) => PayloadType::Extend,
            Payload::Extended(_) => PayloadType::Extended,
            Payload::EstablishIntro(_) => PayloadType::EstablishIntro,
        }
    }
}

pub fn deserialize_payload_with_aes_keys(aes_keys: Vec<Vec<u8>>, buffer: &[u8]) -> Result<Payload> {
    let mut decrypted_buffer = buffer.to_vec();
    for aes_key in aes_keys.iter().rev() {
        if aes_key.len() < 32 {
            return Err(anyhow::anyhow!("AES key is too short {:?}", aes_key));
        }
        decrypted_buffer = decrypt_buffer_with_aes(&aes_key, &decrypted_buffer)?;
    }
    let payload: Payload = serde_json::from_slice(&decrypted_buffer)?;
    info!("Decrypted payload successfully");
    Ok(payload)
}

pub fn decrypt_buffer_with_aes(aes_key: &[u8], buffer: &[u8]) -> Result<Vec<u8>> {
    info!("Decrypting with key {:?}", hex::encode(&aes_key[0..32]));
    if aes_key.len() < 32 {
        return Err(anyhow::anyhow!("AES key is too short {:?}", aes_key));
    }
    let decrypted_buffer = decrypt(Cipher::aes_256_ctr(), &aes_key[0..32], None, buffer)?;
    Ok(decrypted_buffer)
}

pub fn serialize_payload_with_aes_keys(
    aes_keys: Vec<Vec<u8>>,
    payload: &Payload,
) -> Result<Vec<u8>> {
    let mut serialized_payload = serde_json::to_vec(payload)?;
    for aes_key in aes_keys {
        if aes_key.len() < 32 {
            return Err(anyhow::anyhow!("AES key is too short {:?}", aes_key));
        }
        serialized_payload = encrypt_buffer_with_aes(&aes_key, &serialized_payload)?;
    }
    Ok(serialized_payload)
}

pub fn encrypt_buffer_with_aes(aes_key: &[u8], buffer: &[u8]) -> Result<Vec<u8>> {
    info!("Encrypting with key {:?}", hex::encode(&aes_key[0..32]));
    if aes_key.len() < 32 {
        return Err(anyhow::anyhow!("AES key is too short {:?}", aes_key));
    }
    let encrypted_buffer = encrypt(Cipher::aes_256_ctr(), &aes_key[0..32], None, buffer)?;
    Ok(encrypted_buffer)
}

#[cfg(test)]
mod tests {
    use super::{deserialize_payload_with_aes_keys, serialize_payload_with_aes_keys};
    use crate::{
        data::{CreatePayload, OnionSkin},
        Payload,
    };
    use rand::Rng;

    #[test]
    fn test_deserialize_payload_with_aes_keys() {
        let payload = Payload::Create(CreatePayload {
            onion_skin: OnionSkin {
                rsa_encrypted_aes_key: vec![1, 2, 3, 4],
                aes_encrypted_dh_key: vec![5, 6, 7, 8],
            },
        });
        // generate random 32 bytes aes key
        let aes_key_1 = rand::thread_rng().gen::<[u8; 32]>();
        let aes_key_2 = rand::thread_rng().gen::<[u8; 32]>();
        let aes_key_3 = rand::thread_rng().gen::<[u8; 32]>();
        let aes_keys = vec![aes_key_1.to_vec(), aes_key_2.to_vec(), aes_key_3.to_vec()];
        let serialized_payload =
            serialize_payload_with_aes_keys(aes_keys.clone(), &payload).unwrap();
        let deserialized_payload =
            deserialize_payload_with_aes_keys(aes_keys.clone(), &serialized_payload).unwrap();
        assert_eq!(payload, deserialized_payload);
    }
}
