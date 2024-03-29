use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::AesKey;

pub struct Users(Arc<RwLock<HashMap<[u8; 32], (AesKey, u16, u16)>>>);

impl Users {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, address: [u8; 32]) -> Option<(AesKey, u16, u16)> {
        match self.0.read().unwrap().get(&address) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    pub fn insert(&self, address: [u8; 32], aes_key: AesKey, circ_id: u16, stream_id: u16) {
        self.0
            .write_all()
            .unwrap()
            .insert(address, (aes_key, circ_id, stream_id));
    }
}
