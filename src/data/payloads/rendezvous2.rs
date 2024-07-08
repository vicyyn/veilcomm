use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Rendezvous2Payload {
    pub rendezvous_cookie: uuid::Uuid,
    pub dh_key: Vec<u8>,
}
