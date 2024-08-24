use crate::{CircuitId, Logger, RelayId, RendezvousCookieId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct SendDataToRelayBody {
    pub relay_id: RelayId,
    pub rendezvous_cookie: RendezvousCookieId,
    pub circuit_id: CircuitId,
    pub data: Vec<u8>,
}

#[post("/users/{user_id}/send_data")]
pub async fn send_data(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendDataToRelayBody>,
) -> impl Responder {
    let result: Result<()> = async {
        let data_lock = data.lock().await;
        let user = data_lock
            .iter()
            .find(|u| u.user_descriptor.id == *user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        user.send_data(
            body.relay_id,
            body.rendezvous_cookie,
            body.circuit_id,
            body.data.clone(),
        )
        .context("Failed to send data")?;
        Ok(())
    }
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            Logger::error("API", format!("Error in send_data: {}", e));
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}
