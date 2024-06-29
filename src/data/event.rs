use super::PayloadType;
use std::net::SocketAddr;

#[derive(PartialEq, Eq, Debug)]
pub struct Event(pub PayloadType, pub SocketAddr);
