use crate::{Logger, Relay};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct StartRelayBody {
    pub nickname: String,
}

#[post("/start_relay")]
async fn start_relay(
    data: web::Data<Arc<Mutex<Vec<Relay>>>>,
    body: web::Json<StartRelayBody>,
) -> impl Responder {
    Logger::info(
        "API",
        format!("Starting relay with nickname: {}", body.nickname),
    );
    let mut relays = data.lock().await;
    let relay = Relay::new(body.nickname.clone());
    relay.start();
    relays.push(relay);
    HttpResponse::Ok().finish()
}
