use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EstablishIntroPayload {
    pub introduction_id: uuid::Uuid,
}
