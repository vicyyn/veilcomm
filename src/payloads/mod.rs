pub mod create;
pub mod created;
pub mod extend;
pub mod extended;

pub use create::*;
pub use created::*;
pub use extend::*;
pub use extended::*;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug)]
pub enum Payload {
    Create(CreatePayload),
    Created(CreatedPayload),
    Extend(ExtendPayload),
    Extended(ExtendedPayload),
}

#[derive(PartialEq, Eq, Debug)]
pub enum PayloadType {
    Create,
    Created,
    Extend,
    Extended,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Event(pub PayloadType, pub SocketAddr);

impl Payload {
    pub fn get_type(&self) -> PayloadType {
        match self {
            Payload::Create(_) => PayloadType::Create,
            Payload::Created(_) => PayloadType::Created,
            Payload::Extend(_) => PayloadType::Extend,
            Payload::Extended(_) => PayloadType::Extended,
        }
    }
}
