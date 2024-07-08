use serde::{Deserialize, Serialize};

use crate::{relay::RelayDescriptor, OnionSkin};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Introduce1Payload {
    pub stream_id: uuid::Uuid,
    pub introduction_id: uuid::Uuid,
    pub rendezvous_point_descriptor: RelayDescriptor,
    pub rendezvous_cookie: uuid::Uuid,
    pub onion_skin: OnionSkin,
}
