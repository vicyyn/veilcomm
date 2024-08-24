use crate::{CircuitId, Logger, RelayId, StreamId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct SendBeginToRelayBody {
    pub relay_id: RelayId,
    pub circuit_id: CircuitId,
    pub begin_relay_id: RelayId,
}

#[post("/users/{user_id}/send_begin")]
async fn send_begin(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendBeginToRelayBody>,
) -> impl Responder {
    let result: Result<()> = async {
        let data_lock = data.lock().await;
        let user = data_lock
            .iter()
            .find(|u| u.user_descriptor.id == *user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        let stream_id = StreamId::new_v4();
        user.send_begin(
            body.relay_id,
            body.circuit_id,
            stream_id,
            body.begin_relay_id,
        )
        .context("Failed to send begin")?;
        Ok(())
    }
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            Logger::error("API", format!("Error in send_begin: {}", e));
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}
