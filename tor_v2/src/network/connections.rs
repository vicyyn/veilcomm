use std::{net::SocketAddrV4, sync::Arc};

use dashmap::DashMap;

use crate::Connection;

pub struct Connections(Arc<DashMap<SocketAddrV4, Connection>>);

impl Connections {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, socket_address: SocketAddrV4) -> Option<Connection> {
        match self.0.get(&socket_address) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn insert(&self, socket_address: SocketAddrV4, connection: Connection) {
        self.0.insert(socket_address, connection);
    }
}
