use crate::{CircuitId, Logger, RelayId, RendezvousCookieId, User, UserId};
use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct SendEstablishRendezvousBody {
    pub relay_id: RelayId,
    pub circuit_id: CircuitId,
}

#[post("/users/{user_id}/send_establish_rendezvous")]
pub async fn send_establish_rendezvous(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<UserId>,
    body: web::Json<SendEstablishRendezvousBody>,
) -> impl Responder {
    let result: Result<()> = async {
        let data_lock = data.lock().await;
        let user = data_lock
            .iter()
            .find(|u| u.user_descriptor.id == *user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        let rendezvous_cookie = RendezvousCookieId::new_v4();
        user.send_establish_rendezvous(body.relay_id, rendezvous_cookie, body.circuit_id)
            .context("Failed to send establish rendezvous")?;
        Ok(())
    }
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            Logger::error("API", format!("Error in send_establish_rendezvous: {}", e));
            HttpResponse::InternalServerError().json(format!("Internal server error: {}", e))
        }
    }
}
