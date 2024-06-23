use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtendedPayload {
    pub circuit_id: uuid::Uuid,
    pub dh_key: Vec<u8>,
}
