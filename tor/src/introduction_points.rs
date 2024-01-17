use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct IntroductionPoints(Arc<RwLock<HashMap<[u8; 32], u16>>>);

impl Default for IntroductionPoints {
    fn default() -> Self {
        Self::new()
    }
}

impl IntroductionPoints {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, address: [u8; 32]) -> Option<u16> {
        self.0.read().unwrap().get(&address).copied()
    }

    pub fn insert(&self, address: [u8; 32], circuit_id: u16) {
        self.0.write().unwrap().insert(address, circuit_id);
    }
}
