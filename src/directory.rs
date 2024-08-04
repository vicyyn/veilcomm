use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::relay::RelayDescriptor;
use crate::user::UserDescriptor;
use crate::Logger;

pub struct Directory {
    relays: Arc<Mutex<Vec<RelayDescriptor>>>,
    users: Arc<Mutex<Vec<UserDescriptor>>>,
    address: SocketAddr,
}

impl Directory {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            relays: Arc::new(Mutex::new(Vec::new())),
            users: Arc::new(Mutex::new(Vec::new())),
            address,
        }
    }

    pub fn start(&self) {
        let relays = self.relays.clone();
        let users = self.users.clone();
        let address = self.address;
        tokio::spawn(async move {
            Logger::info("Directory", format!("Starting HTTP server at {}", address));
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(relays.clone()))
                    .app_data(web::Data::new(users.clone()))
                    .service(get_relays)
                    .service(publish_relay)
                    .service(get_users)
                    .service(publish_user)
                    .service(update_user_introduction_points)
            })
            .disable_signals()
            .bind(address)
            .unwrap_or_else(|_| panic!("Could not bind server to address {}", address))
            .run()
            .await
            .unwrap_or_else(|e| Logger::error("Directory", format!("Error: {}", e)));
        });
    }
}

#[get("/relays")]
async fn get_relays(data: web::Data<Arc<Mutex<Vec<RelayDescriptor>>>>) -> impl Responder {
    Logger::info("Directory", "Fetching all relays");
    let relays = data.lock().await;
    HttpResponse::Ok().json(&*relays)
}

#[post("/relays")]
async fn publish_relay(
    relay: web::Json<RelayDescriptor>,
    data: web::Data<Arc<Mutex<Vec<RelayDescriptor>>>>,
) -> impl Responder {
    Logger::info("Directory", "Publishing a new relay");
    let mut relays = data.lock().await;
    relays.push(relay.into_inner());
    HttpResponse::Ok().finish()
}

#[get("/users")]
async fn get_users(data: web::Data<Arc<Mutex<Vec<UserDescriptor>>>>) -> impl Responder {
    Logger::info("Directory", "Fetching all users");
    let users = data.lock().await;
    HttpResponse::Ok().json(&*users)
}

#[post("/users/{user_id}/introduction_points")]
async fn update_user_introduction_points(
    user_id: web::Path<Uuid>,
    introduction_points: web::Json<Vec<(Uuid, SocketAddr)>>,
    data: web::Data<Arc<Mutex<Vec<UserDescriptor>>>>,
) -> impl Responder {
    Logger::info(
        "Directory",
        format!("Updating introduction points for user {}", user_id),
    );
    let mut users = data.lock().await;
    for user in users.iter_mut() {
        if user.id == *user_id {
            user.introduction_points = introduction_points.into_inner();
            return HttpResponse::Ok().finish();
        }
    }
    HttpResponse::NotFound().finish()
}

#[post("/users")]
async fn publish_user(
    user: web::Json<UserDescriptor>,
    data: web::Data<Arc<Mutex<Vec<UserDescriptor>>>>,
) -> impl Responder {
    Logger::info("Directory", "Publishing a new user");
    let mut users = data.lock().await;
    users.push(user.into_inner());
    HttpResponse::Ok().finish()
}
