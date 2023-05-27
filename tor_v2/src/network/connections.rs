use std::{
    net::{SocketAddrV4, TcpStream},
    sync::Arc,
};

use dashmap::DashMap;

pub struct Connections(Arc<DashMap<SocketAddrV4, TcpStream>>);

impl Connections {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, socket_address: &SocketAddrV4) -> Option<TcpStream> {
        match self.0.get(socket_address) {
            Some(v) => v.try_clone().ok(),
            None => None,
        }
    }

    pub fn insert(&self, socket_address: SocketAddrV4, tcp_stream: TcpStream) {
        self.0.insert(socket_address, tcp_stream);
    }
}
