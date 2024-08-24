use crate::{CircuitId, RelayId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct SendEstablishIntroductionBody {
    pub relay_id: RelayId,
    pub circuit_id: CircuitId,
}

#[post("/users/{user_id}/send_establish_introduction")]
async fn send_establish_introduction(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendEstablishIntroductionBody>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    let introduction_id = UserId::new_v4();
    user.send_establish_introduction(body.relay_id, introduction_id, body.circuit_id)
        .unwrap();
    HttpResponse::Ok().finish()
}
