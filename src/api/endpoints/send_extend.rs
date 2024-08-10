use crate::{CircuitId, RelayId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct SendExtendBody {
    pub relay_id: RelayId,
    pub circuit_id: CircuitId,
    pub extend_to: RelayId,
}

#[post("/users/{user_id}/send_extend_to_relay")]
async fn send_extend_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendExtendBody>,
) -> impl Responder {
    let data_lock = data.lock().unwrap();
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_extend_to_relay(body.relay_id, body.extend_to, body.circuit_id)
        .unwrap();
    HttpResponse::Ok().finish()
}
