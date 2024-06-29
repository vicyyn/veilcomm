use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ExtendedPayload {
    pub address: SocketAddr,
    pub dh_key: Vec<u8>,
}
