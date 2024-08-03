use crate::OnionSkin;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CreatePayload {
    pub onion_skin: OnionSkin,
}
