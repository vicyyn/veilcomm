use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CreatedPayload {
    pub dh_key: Vec<u8>,
}
