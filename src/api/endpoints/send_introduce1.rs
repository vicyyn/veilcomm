use crate::{CircuitId, IntroductionPointId, RelayId, RendezvousCookieId, StreamId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct SendIntroduce1Body {
    pub relay_id: RelayId,
    pub introduction_id: IntroductionPointId,
    pub stream_id: StreamId,
    pub rendezvous_point_relay_id: RelayId,
    pub rendezvous_cookie: RendezvousCookieId,
    pub introduction_rsa_public: Vec<u8>,
    pub circuit_id: CircuitId,
}

#[post("/users/{user_id}/send_introduce1_to_relay")]
async fn send_introduce1_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendIntroduce1Body>,
) -> impl Responder {
    let data_lock = data.lock().unwrap();
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_introduce1_to_relay(
        body.relay_id,
        body.introduction_id,
        body.stream_id,
        body.rendezvous_point_relay_id,
        body.rendezvous_cookie,
        body.introduction_rsa_public.clone(),
        body.circuit_id,
    )
    .unwrap();
    HttpResponse::Ok().finish()
}
