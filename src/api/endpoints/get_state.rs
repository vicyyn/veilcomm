use crate::{
    CircuitId, Handshake, IntroductionPointId, Logger, Relay, RelayId, RendezvousCookieId,
    StreamId, User, UserId,
};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
use serde::Deserialize;

#[cfg_attr(test, derive(Deserialize))]
#[derive(Serialize, Debug)]
pub struct UserState {
    pub id: UserId,
    pub nickname: String,
    pub rsa_public_key: Vec<u8>,
    pub introduction_points: HashMap<IntroductionPointId, RelayId>,
    pub circuits: HashMap<CircuitId, Vec<RelayId>>,
    pub handshakes: HashMap<RelayId, Handshake>,
    pub rendezvous_cookies: HashMap<RendezvousCookieId, RelayId>,
    pub connected_users: HashMap<RendezvousCookieId, Handshake>,
    pub streams: HashMap<StreamId, RelayId>,
    pub logs: Vec<String>,
}

#[cfg_attr(test, derive(Deserialize))]
#[derive(Serialize, Debug)]
pub struct RelayState {
    pub id: RelayId,
    pub is_rendezvous_point: bool,
    pub is_introduction_point: bool,
    pub nickname: String,
    pub circuits: HashMap<CircuitId, RelayId>,
    pub streams: HashMap<StreamId, RelayId>,
    pub logs: Vec<String>,
}

#[cfg_attr(test, derive(Deserialize))]
#[derive(Serialize, Debug)]
pub struct ApiState {
    pub relay_states: Vec<RelayState>,
    pub user_states: Vec<UserState>,
}

#[get("/get_state")]
async fn get_state(
    relays: web::Data<Arc<Mutex<Vec<Relay>>>>,
    users: web::Data<Arc<Mutex<Vec<User>>>>,
) -> impl Responder {
    Logger::info("API", "GET /get_state");
    let relays_lock = relays.lock().await;
    let users_lock = users.lock().await;
    let api_state = ApiState {
        relay_states: relays_lock.iter().map(|relay| relay.get_state()).collect(),
        user_states: users_lock.iter().map(|user| user.get_state()).collect(),
    };
    HttpResponse::Ok().json(api_state)
}
