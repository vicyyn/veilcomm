use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatedPayload {
    pub circuit_id: Uuid,
    pub dh_key: Vec<u8>,
}
