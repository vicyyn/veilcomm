use std::{
    collections::HashMap,
    net::SocketAddrV4,
    sync::{Arc, RwLock},
};

pub struct Streams(Arc<RwLock<HashMap<u16, SocketAddrV4>>>);

impl Streams {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn get(&self, id: u16) -> Option<SocketAddrV4> {
        if let Some(socket_address) = self.0.read().unwrap().get(&id) {
            return Some(socket_address.clone());
        }
        None
    }

    pub fn insert(&self, id: u16, socket_address: SocketAddrV4) {
        self.0.write_all().unwrap().insert(id, socket_address);
    }
}

impl Clone for Streams {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
