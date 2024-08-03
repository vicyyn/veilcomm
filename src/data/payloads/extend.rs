use crate::OnionSkin;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ExtendPayload {
    pub address: SocketAddr,
    pub onion_skin: OnionSkin,
}
