use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::relay::RelayDescriptor;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct BeginPayload {
    pub stream_id: Uuid,
    pub relay_descriptor: RelayDescriptor,
}
