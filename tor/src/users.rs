use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::AESKey;

#[allow(clippy::type_complexity)]
#[derive(Default, Clone)]
pub struct Users(Arc<RwLock<HashMap<[u8; 32], (AESKey, u16, u16)>>>);

impl Users {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
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
