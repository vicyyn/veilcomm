use crate::{CircuitId, Logger, RelayId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::Result;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    Logger::info("API", format!("send_create: {:?}", user_id));

    let result: Result<()> = async {
        let data_lock = data.lock().await;
        let circuit_id = CircuitId::new_v4();
        let user = data_lock
            .iter()
            .find(|u| u.user_descriptor.id == *user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        user.send_create(body.relay_id, circuit_id)
            .map_err(|e| anyhow::anyhow!("Failed to send create: {}", e))?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            Logger::error("API", format!("Error in send_create: {}", e));
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}
