use crate::OnionSkin;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ExtendPayload {
    pub extend_to: Uuid,
    pub onion_skin: OnionSkin,
}
