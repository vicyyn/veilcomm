use crate::{CircuitId, RelayId, RendezvousCookieId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct SendEstablishRendezvousBody {
    pub relay_id: RelayId,
    pub circuit_id: CircuitId,
}

#[post("/users/{user_id}/send_establish_rendezvous")]
async fn send_establish_rendezvous(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendEstablishRendezvousBody>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    let rendezvous_cookie = RendezvousCookieId::new_v4();
    user.send_establish_rendezvous(body.relay_id, rendezvous_cookie, body.circuit_id)
        .unwrap();
    HttpResponse::Ok().finish()
}
