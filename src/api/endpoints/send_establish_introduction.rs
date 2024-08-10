use crate::{CircuitId, RelayId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct SendEstablishIntroductionBody {
    pub relay_socket: RelayId,
    pub circuit_id: CircuitId,
}

#[post("/users/{user_id}/send_establish_introduction_to_relay")]
async fn send_establish_introduction_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendEstablishIntroductionBody>,
) -> impl Responder {
    let data_lock = data.lock().unwrap();
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    let introduction_id = UserId::new_v4();
    user.send_establish_introduction_to_relay(body.relay_socket, introduction_id, body.circuit_id)
        .unwrap();
    HttpResponse::Ok().json(introduction_id.to_string())
}
