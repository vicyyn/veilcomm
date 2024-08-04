use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

pub fn directory_address() -> SocketAddr {}

pub fn api_address() -> SocketAddr {
    SocketAddr::from_str("127.0.0.1:8081").unwrap()
}

pub type Connections = Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<OwnedWriteHalf>>>>>;
