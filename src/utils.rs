use std::{net::SocketAddr, str::FromStr};
use uuid::Uuid;

pub fn api_address() -> SocketAddr {}

pub type CircuitId = Uuid;
pub type RelayId = Uuid;
pub type UserId = Uuid;
pub type IntroductionPointId = Uuid;
pub type RendezvousCookieId = Uuid;
pub type StreamId = Uuid;
pub type Handshake = Vec<u8>;
