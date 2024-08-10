use crate::{CircuitId, RelayId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct EstablishCircuitBody {
    pub circuit_id: CircuitId,
    pub relay_address_1: RelayId,
    pub relay_address_2: RelayId,
    pub relay_address_3: RelayId,
}

#[post("/users/{user_id}/establish_circuit")]
async fn establish_circuit(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<EstablishCircuitBody>,
) -> impl Responder {
    let data_lock = data.lock().unwrap();
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.establish_circuit(
        body.circuit_id,
        body.relay_address_1,
        body.relay_address_2,
        body.relay_address_3,
    )
    .unwrap();
    HttpResponse::Ok().finish()
}
