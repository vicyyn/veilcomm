// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.

use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub const CELL_SIZE: usize = 512;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cell {
    pub circ_id: u16,
    pub command: u8,
    #[serde(with = "BigArray")]
    pub payload: [u8; CELL_SIZE - 3],
}

impl Cell {
    pub fn new(circ_id: u16, command: u8, payload: [u8; CELL_SIZE - 3]) -> Self {
        Self {
            circ_id,
            command,
            payload,
        }
    }

    pub fn encrypt(&self, aes_key: &[u8]) -> Cell {
        Self {
            circ_id: self.circ_id,
            command: self.command,
            payload: encrypt(Cipher::aes_128_ctr(), aes_key, None, &self.payload)
                .unwrap()
                .try_into()
                .unwrap(),
        }
    }

    pub fn decrypt(&self, aes_key: &[u8]) -> Cell {
        Self {
            circ_id: self.circ_id,
            command: self.command,
            payload: decrypt(Cipher::aes_128_ctr(), aes_key, None, &self.payload)
                .unwrap()
                .try_into()
                .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_cell_encryption_decryption() {
        let cell = Cell::new(0, 0, [0; CELL_SIZE - 3]);
        let aes_key = [0; 16];
        let encrypted_cell = cell.encrypt(&aes_key);
        assert_eq!(encrypted_cell.decrypt(&aes_key), cell);
    }
}
