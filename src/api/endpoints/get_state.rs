use crate::{
    CircuitId, Handshake, IntroductionPointId, Relay, RelayId, RendezvousCookieId, StreamId, User,
    UserId,
};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(test)]
use serde::Deserialize;

#[cfg_attr(test, derive(Deserialize))]
#[derive(Serialize)]
pub struct UserState {
    pub id: UserId,
    pub nickname: String,
    pub introduction_points: HashMap<IntroductionPointId, RelayId>,
    pub circuits: HashMap<CircuitId, Vec<RelayId>>,
    pub handshakes: HashMap<RelayId, Handshake>,
    pub connected_users: HashMap<RendezvousCookieId, Handshake>,
    pub streams: HashMap<StreamId, RelayId>,
    pub logs: Vec<String>,
}

#[cfg_attr(test, derive(Deserialize))]
#[derive(Serialize)]
pub struct RelayState {
    pub id: RelayId,
    pub nickname: String,
    pub circuits: HashMap<CircuitId, RelayId>,
    pub logs: Vec<String>,
}

#[cfg_attr(test, derive(Deserialize))]
#[derive(Serialize)]
pub struct ApiState {
    pub relay_states: Vec<RelayState>,
    pub user_states: Vec<UserState>,
}

#[post("/get_state")]
async fn get_state(
    relays: web::Data<Arc<Mutex<Vec<Relay>>>>,
    users: web::Data<Arc<Mutex<Vec<User>>>>,
) -> impl Responder {
    let relays_lock = relays.lock().unwrap();
    let users_lock = users.lock().unwrap();

    let api_state = ApiState {
        relay_states: relays_lock.iter().map(|relay| relay.get_state()).collect(),
        user_states: users_lock.iter().map(|user| user.get_state()).collect(),
    };

    HttpResponse::Ok().json(api_state)
}
