use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EstablishIntroductionPayload {
    pub rsa_publickey: Vec<u8>,
    pub introduction_id: uuid::Uuid,
}
