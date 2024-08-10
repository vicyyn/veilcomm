use crate::{CircuitId, RelayId, RendezvousCookieId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct SendRendezvous1Body {
    pub relay_id: RelayId,
    pub rendezvous_cookie: RendezvousCookieId,
    pub circuit_id: CircuitId,
}

#[post("/users/{user_id}/send_rendezvous1_to_relay")]
async fn send_rendezvous1_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendRendezvous1Body>,
) -> impl Responder {
    let data_lock = data.lock().unwrap();
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_rendezvous1_to_relay(body.relay_id, body.rendezvous_cookie, body.circuit_id)
        .unwrap();
    HttpResponse::Ok().finish()
}
