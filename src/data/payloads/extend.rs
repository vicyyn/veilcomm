use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::OnionSkin;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ExtendPayload {
    pub address: SocketAddr,
    pub onion_skin: OnionSkin,
}
