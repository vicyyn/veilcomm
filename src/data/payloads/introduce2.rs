use serde::{Deserialize, Serialize};

use crate::{relay::RelayDescriptor, OnionSkin};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Introduce2Payload {
    pub rendezvous_point_descriptor: RelayDescriptor,
    pub rendezvous_cookie: uuid::Uuid,
    pub onion_skin: OnionSkin,
}
