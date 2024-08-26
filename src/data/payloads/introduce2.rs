use crate::OnionSkin;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Introduce2Payload {
    pub rendezvous_cookie: uuid::Uuid,
    pub onion_skin: OnionSkin,
}
