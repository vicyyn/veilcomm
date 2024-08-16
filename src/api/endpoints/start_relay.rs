use crate::Relay;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct StartRelayBody {
    pub nickname: String,
}

#[post("/start_relay")]
async fn start_relay(
    data: web::Data<Arc<Mutex<Vec<Relay>>>>,
    body: web::Json<StartRelayBody>,
) -> impl Responder {
    let mut relays = data.lock().unwrap();
    let relay = Relay::new(body.nickname.clone());
    relay.start();
    relays.push(relay);
    HttpResponse::Ok().finish()
}
