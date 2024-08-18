use crate::{Logger, User};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct StartUserBody {
    pub nickname: String,
}

#[post("/start_user")]
async fn start_user(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    body: web::Json<StartUserBody>,
) -> impl Responder {
    Logger::info(
        "API",
        format!("Starting user with nickname: {}", body.nickname),
    );
    let mut users = data.lock().await;
    let user = User::new(body.nickname.clone());
    user.start();
    users.push(user);
    HttpResponse::Ok().finish()
}
