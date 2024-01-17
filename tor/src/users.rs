use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::AESKey;

pub struct Users(Arc<RwLock<HashMap<[u8; 32], (AESKey, u16, u16)>>>);

impl Default for Users {
    fn default() -> Self {
        Self::new()
    }
}

impl Users {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, address: [u8; 32]) -> Option<(AESKey, u16, u16)> {
        self.0.read().unwrap().get(&address).copied()
    }

    pub fn insert(&self, address: [u8; 32], aes_key: AESKey, circ_id: u16, stream_id: u16) {
        self.0
            .write()
            .unwrap()
            .insert(address, (aes_key, circ_id, stream_id));
    }
}
