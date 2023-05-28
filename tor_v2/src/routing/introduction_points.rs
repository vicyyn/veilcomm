use std::sync::Arc;

use dashmap::DashMap;

#[derive(Debug)]
pub struct IntroductionPoints(Arc<DashMap<[u8; 32], u16>>);

impl IntroductionPoints {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, address: [u8; 32]) -> Option<u16> {
        match self.0.get(&address) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    pub fn insert(&self, address: [u8; 32], circuit_id: u16) {
        self.0.insert(address, circuit_id);
    }
}
