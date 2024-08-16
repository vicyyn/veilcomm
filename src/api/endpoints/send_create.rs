use crate::{CircuitId, RelayId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct SendCreateBody {
    pub relay_id: RelayId,
}

#[post("/users/{user_id}/send_create")]
async fn send_create(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendCreateBody>,
) -> impl Responder {
    let data_lock = data.lock().unwrap();
    let circuit_id = CircuitId::new_v4();
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_create(body.relay_id, circuit_id).unwrap();
    HttpResponse::Ok().json(circuit_id.to_string())
}
