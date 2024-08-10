use crate::{
    establish_circuit, get_state, send_begin_to_relay, send_create_to_relay, send_data_to_relay,
    send_establish_introduction_to_relay, send_establish_rendezvous_to_relay, send_extend_to_relay,
    send_introduce1_to_relay, send_rendezvous1_to_relay, start_relay, start_user, Relay, User,
};
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use log::{error, info};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Api {
    relays: Arc<Mutex<Vec<Relay>>>,
    users: Arc<Mutex<Vec<User>>>,
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
            info!("Starting API HTTP server at {}", address);

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
                    .service(send_create_to_relay)
                    .service(send_extend_to_relay)
                    .service(establish_circuit)
                    .service(send_establish_rendezvous_to_relay)
                    .service(send_begin_to_relay)
                    .service(send_introduce1_to_relay)
                    .service(send_data_to_relay)
                    .service(send_establish_introduction_to_relay)
                    .service(send_establish_rendezvous_to_relay)
                    .service(send_rendezvous1_to_relay)
                    .service(get_state)
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
