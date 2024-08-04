use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ExtendedPayload {
    pub extend_to: Uuid,
    pub dh_key: Vec<u8>,
}
