use super::PayloadType;
use uuid::Uuid;

#[derive(PartialEq, Eq, Debug)]
pub struct Event(pub PayloadType, pub Uuid);
