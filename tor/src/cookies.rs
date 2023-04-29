use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Hash, PartialEq, Eq)]
pub struct Cookie(pub [u8; 20]);

impl From<[u8; 20]> for Cookie {
    fn from(value: [u8; 20]) -> Self {
        Self(value)
    }
}

pub struct Cookies(Arc<RwLock<HashMap<Cookie, u16>>>);

impl Cookies {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, cookie: Cookie) -> Option<u16> {
        match self.0.read().unwrap().get(&cookie) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    pub fn insert(&self, cookie: Cookie, circuit_id: u16) {
        self.0.write().unwrap().insert(cookie, circuit_id);
    }
}
