use anyhow::Result;
use openssl::symm::{decrypt, encrypt, Cipher};
use rand::{thread_rng, Rng};

pub fn generate_random_aes_key() -> [u8; 16] {
    let mut rand = thread_rng();
    rand.gen::<[u8; 16]>()
}

pub fn decrypt_buffer_with_aes(aes_key: &[u8], buffer: &[u8]) -> Result<Vec<u8>> {
    if aes_key.len() < 32 {
        return Err(anyhow::anyhow!("AES key is too short {:?}", aes_key));
    }
    let decrypted_buffer = decrypt(Cipher::aes_256_ctr(), &aes_key[0..32], None, buffer)?;
    Ok(decrypted_buffer)
}

pub fn encrypt_buffer_with_aes(aes_key: &[u8], buffer: &[u8]) -> Result<Vec<u8>> {
    if aes_key.len() < 32 {
        return Err(anyhow::anyhow!("AES key is too short {:?}", aes_key));
    }
    let encrypted_buffer = encrypt(Cipher::aes_256_ctr(), &aes_key[0..32], None, buffer)?;
    Ok(encrypted_buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_aes_key() {
        let key1 = generate_random_aes_key();
        let key2 = generate_random_aes_key();

        assert_eq!(key1.len(), 16);
        assert_eq!(key2.len(), 16);
        assert_ne!(key1, key2); // Ensure randomness (there's a tiny chance this could fail)
    }

    #[test]
    fn test_encrypt_decrypt_buffer_with_aes() -> Result<()> {
        let aes_key = [0u8; 32]; // Use a 32-byte key for AES-256
        let original_data = b"Hello, World!";

        let encrypted = encrypt_buffer_with_aes(&aes_key, original_data)?;
        let decrypted = decrypt_buffer_with_aes(&aes_key, &encrypted)?;

        assert_ne!(encrypted, original_data);
        assert_eq!(decrypted, original_data);

        Ok(())
    }

    #[test]
    fn test_encrypt_decrypt_with_random_key() -> Result<()> {
        let aes_key = generate_random_aes_key();
        let extended_key = [aes_key, aes_key].concat(); // Extend to 32 bytes for AES-256
        let original_data = b"Random key test";

        let encrypted = encrypt_buffer_with_aes(&extended_key, original_data)?;
        let decrypted = decrypt_buffer_with_aes(&extended_key, &encrypted)?;

        assert_ne!(encrypted, original_data);
        assert_eq!(decrypted, original_data);

        Ok(())
    }

    #[test]
    fn test_short_key_error() {
        let short_key = [0u8; 15];
        let data = b"Test data";

        let encrypt_result = encrypt_buffer_with_aes(&short_key, data);
        assert!(encrypt_result.is_err());

        let decrypt_result = decrypt_buffer_with_aes(&short_key, data);
        assert!(decrypt_result.is_err());
    }

    #[test]
    fn test_different_keys_produce_different_ciphertexts() -> Result<()> {
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let data = b"Same data, different keys";

        let encrypted1 = encrypt_buffer_with_aes(&key1, data)?;
        let encrypted2 = encrypt_buffer_with_aes(&key2, data)?;

        assert_ne!(encrypted1, encrypted2);

        Ok(())
    }
}
