use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelayCell {
    pub circuit_id: Uuid,
    pub payload: Vec<u8>,
}
