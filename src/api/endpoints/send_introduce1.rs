use crate::{
    CircuitId, IntroductionPointId, Logger, RelayId, RendezvousCookieId, StreamId, User, UserId,
};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct SendIntroduce1Body {
    pub relay_id: RelayId,
    pub introduction_id: IntroductionPointId,
    pub stream_id: StreamId,
    pub rendezvous_cookie: RendezvousCookieId,
    pub introduction_rsa_public: Vec<u8>,
    pub circuit_id: CircuitId,
}

#[post("/users/{user_id}/send_introduce1")]
pub async fn send_introduce1(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendIntroduce1Body>,
) -> impl Responder {
    let result: Result<()> = async {
        let data_lock = data.lock().await;
        let user = data_lock
            .iter()
            .find(|u| u.user_descriptor.id == *user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        user.send_introduce1(
            body.relay_id,
            body.introduction_id,
            body.stream_id,
            body.rendezvous_cookie,
            body.introduction_rsa_public.clone(),
            body.circuit_id,
        )
        .context("Failed to send introduce1")?;
        Ok(())
    }
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            Logger::error("API", format!("Error in send_introduce1: {}", e));
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}
