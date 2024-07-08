use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EstablishRendezvousPayload {
    pub rendezvous_cookie: uuid::Uuid,
}
