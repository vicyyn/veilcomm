use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::OnionSkin;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtendPayload {
    pub circuit_id: uuid::Uuid,
    pub address: SocketAddr,
    pub onion_skin: OnionSkin,
}
