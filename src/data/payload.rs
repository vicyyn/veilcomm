use super::*;
use anyhow::Result;
use log::info;
use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Payload {
    EstablishRendezvous(EstablishRendezvousPayload),
    EstablishedRendezvous(EstablishedRendezvousPayload),
    EstablishIntroduction(EstablishIntroductionPayload),
    EstablishedIntroduction(EstablishedIntroductionPayload),
    Create(CreatePayload),
    Created(CreatedPayload),
    Extend(ExtendPayload),
    Extended(ExtendedPayload),
    Begin(BeginPayload),
    Connected(ConnectedPayload),
    Introduce1(Introduce1Payload),
    Introduce2(Introduce2Payload),
    IntroduceAck(IntroduceAckPayload),
    Rendezvous1(Rendezvous1Payload),
    Rendezvous2(Rendezvous2Payload),
    Data(DataPayload),
}

#[derive(PartialEq, Eq, Debug)]
pub enum PayloadType {
    Create,
    Created,
    Extend,
    Extended,
    EstablishRendezvous,
    EstablishedRendezvous,
    EstablishIntroduction,
    EstablishedIntroduction,
    Begin,
    Connected,
    Introduce1,
    Introduce2,
    IntroduceAck,
    Rendezvous1,
    Rendezvous2,
    Data,
}

impl Payload {
    pub fn get_type(&self) -> PayloadType {
        match self {
            Payload::Create(_) => PayloadType::Create,
            Payload::Created(_) => PayloadType::Created,
            Payload::Extend(_) => PayloadType::Extend,
            Payload::Extended(_) => PayloadType::Extended,
            Payload::EstablishRendezvous(_) => PayloadType::EstablishRendezvous,
            Payload::EstablishedRendezvous(_) => PayloadType::EstablishedRendezvous,
            Payload::EstablishIntroduction(_) => PayloadType::EstablishIntroduction,
            Payload::EstablishedIntroduction(_) => PayloadType::EstablishedIntroduction,
            Payload::Begin(_) => PayloadType::Begin,
            Payload::Connected(_) => PayloadType::Connected,
            Payload::Introduce1(_) => PayloadType::Introduce1,
            Payload::Introduce2(_) => PayloadType::Introduce2,
            Payload::IntroduceAck(_) => PayloadType::IntroduceAck,
            Payload::Rendezvous1(_) => PayloadType::Rendezvous1,
            Payload::Rendezvous2(_) => PayloadType::Rendezvous2,
            Payload::Data(_) => PayloadType::Data,
        }
    }
}

pub fn deserialize_payload_with_aes_keys(aes_keys: Vec<Vec<u8>>, buffer: &[u8]) -> Result<Payload> {
    let mut decrypted_buffer = buffer.to_vec();
    for aes_key in aes_keys {
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
    for aes_key in aes_keys.iter().rev() {
        if aes_key.len() < 32 {
            return Err(anyhow::anyhow!("AES key is too short {:?}", aes_key));
        }
        serialized_payload = encrypt_buffer_with_aes(aes_key, &serialized_payload)?;
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
    use super::*;
    use rand::Rng;

    #[test]
    fn test_encrypt_decrypt_buffer_with_aes() {
        let aes_key: [u8; 32] = rand::thread_rng().gen();
        let buffer = b"Hello, World!";

        let encrypted = encrypt_buffer_with_aes(&aes_key, buffer).unwrap();
        let decrypted = decrypt_buffer_with_aes(&aes_key, &encrypted).unwrap();

        assert_eq!(buffer.to_vec(), decrypted);
    }

    #[test]
    fn test_serialize_deserialize_payload_with_multiple_aes_keys() {
        let payload = Payload::Create(CreatePayload {
            onion_skin: OnionSkin {
                rsa_encrypted_aes_key: vec![1, 2, 3, 4],
                aes_encrypted_dh_key: vec![5, 6, 7, 8],
            },
        });

        let aes_keys: Vec<Vec<u8>> = (0..3)
            .map(|_| rand::thread_rng().gen::<[u8; 32]>().to_vec())
            .collect();

        let serialized = serialize_payload_with_aes_keys(aes_keys.clone(), &payload).unwrap();
        let deserialized = deserialize_payload_with_aes_keys(aes_keys, &serialized).unwrap();

        assert_eq!(payload, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_payload_with_single_aes_key() {
        let payload = Payload::Create(CreatePayload {
            onion_skin: OnionSkin {
                rsa_encrypted_aes_key: vec![1, 2, 3, 4],
                aes_encrypted_dh_key: vec![5, 6, 7, 8],
            },
        });

        let aes_key = rand::thread_rng().gen::<[u8; 32]>().to_vec();
        let aes_keys = vec![aes_key];

        let serialized = serialize_payload_with_aes_keys(aes_keys.clone(), &payload).unwrap();
        let deserialized = deserialize_payload_with_aes_keys(aes_keys, &serialized).unwrap();

        assert_eq!(payload, deserialized);
    }

    #[test]
    #[should_panic(expected = "AES key is too short")]
    fn test_encrypt_with_short_aes_key() {
        let short_key = vec![1, 2, 3];
        let buffer = b"Hello, World!";
        encrypt_buffer_with_aes(&short_key, buffer).unwrap();
    }

    #[test]
    #[should_panic(expected = "AES key is too short")]
    fn test_decrypt_with_short_aes_key() {
        let short_key = vec![1, 2, 3];
        let buffer = vec![1, 2, 3, 4, 5];
        decrypt_buffer_with_aes(&short_key, &buffer).unwrap();
    }

    #[test]
    #[should_panic(expected = "AES key is too short")]
    fn test_serialize_payload_with_short_aes_key() {
        let payload = Payload::Create(CreatePayload {
            onion_skin: OnionSkin {
                rsa_encrypted_aes_key: vec![1, 2, 3, 4],
                aes_encrypted_dh_key: vec![5, 6, 7, 8],
            },
        });
        let short_key = vec![1, 2, 3];
        serialize_payload_with_aes_keys(vec![short_key], &payload).unwrap();
    }

    #[test]
    #[should_panic(expected = "AES key is too short")]
    fn test_deserialize_payload_with_short_aes_key() {
        let short_key = vec![1, 2, 3];
        let buffer = vec![1, 2, 3, 4, 5];
        deserialize_payload_with_aes_keys(vec![short_key], &buffer).unwrap();
    }
}
