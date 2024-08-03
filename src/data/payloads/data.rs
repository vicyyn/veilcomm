use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DataPayload {
    pub data: Vec<u8>,
    pub rendezvous_cookie: uuid::Uuid,
}
