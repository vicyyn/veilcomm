use crate::send_establish_introduction::send_establish_introduction;
use crate::send_establish_rendezvous::send_establish_rendezvous;
use crate::{
    establish_circuit, get_state, send_create, send_data, send_extend, send_introduce1,
    send_rendezvous1, start_relay, start_user, Logger, Relay, User,
};
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use log::error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::send_begin::send_begin;

pub struct Api {
    relays: Arc<Mutex<Vec<Relay>>>,
    users: Arc<Mutex<Vec<User>>>,
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

impl Api {
    pub fn new() -> Self {
        Self {
            relays: Arc::new(Mutex::new(Vec::new())),
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&self) {
        let relays = self.relays.clone();
        let users = self.users.clone();
        let address = SocketAddr::from_str("127.0.0.1:8081").unwrap();
        tokio::spawn(async move {
            Logger::info("API", format!("Starting API HTTP server at {}", address));
            HttpServer::new(move || {
                let cors = Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600);

                App::new()
                    .wrap(cors)
                    .app_data(web::Data::new(relays.clone()))
                    .app_data(web::Data::new(users.clone()))
                    .service(start_relay)
                    .service(start_user)
                    .service(send_create)
                    .service(send_extend)
                    .service(establish_circuit)
                    .service(send_introduce1)
                    .service(send_data)
                    .service(send_rendezvous1)
                    .service(get_state)
                    .service(send_establish_introduction)
                    .service(send_establish_rendezvous)
                    .service(send_begin)
            })
            .disable_signals()
            .bind(address)
            .unwrap_or_else(|_| panic!("Could not bind server to address {}", address))
            .run()
            .await
            .unwrap_or_else(|e| error!("HTTP server error: {}", e));
        });
    }
}
