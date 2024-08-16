use crate::send_establish_introduction::send_establish_introduction;
use crate::send_establish_rendezvous::send_establish_rendezvous;
use crate::{
    establish_circuit, get_state, send_create, send_data_to_relay, send_extend, send_introduce1,
    send_rendezvous1, start_relay, start_user, Relay, User,
};
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use log::{error, info};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

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
                    .service(send_create)
                    .service(send_extend)
                    .service(establish_circuit)
                    .service(send_introduce1)
                    .service(send_data_to_relay)
                    .service(send_rendezvous1)
                    .service(get_state)
                    .service(send_establish_introduction)
                    .service(send_establish_rendezvous)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        send_begin, send_establish_introduction, send_establish_rendezvous, CircuitId,
        StartUserBody,
    };
    use actix_web::{test, App};
    use serde_json::json;
    use uuid::Uuid;

    fn setup_test_environment(
        num_users: usize,
        num_relays: usize,
    ) -> (Arc<Mutex<Vec<User>>>, Arc<Mutex<Vec<Relay>>>) {
        let mut users = Vec::new();
        let mut relays = Vec::new();

        for i in 0..num_users {
            let user = User::new(format!("test_user_{}", i));
            user.start();
            users.push(user);
        }

        for i in 0..num_relays {
            let relay = Relay::new(format!("test_relay_{}", i));
            relay.start();
            relays.push(relay);
        }

        (Arc::new(Mutex::new(users)), Arc::new(Mutex::new(relays)))
    }

    #[actix_rt::test]
    async fn test_start_user() {
        let users: Arc<Mutex<Vec<User>>> = Arc::new(Mutex::new(Vec::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(users.clone()))
                .service(start_user),
        )
        .await;

        let json_string = json!({"nickname": "test_user"}).to_string();

        let req = test::TestRequest::post()
            .uri("/start_user")
            .set_payload(json_string.clone())
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let users_lock = users.lock().unwrap();
        assert_eq!(users_lock.len(), 1);
        assert_eq!(users_lock[0].user_descriptor.nickname, "test_user");

        let body: StartUserBody = serde_json::from_str(&json_string).unwrap();
        assert_eq!(body.nickname, "test_user");
    }

    #[actix_rt::test]
    async fn test_start_relay() {
        let relays: Arc<Mutex<Vec<Relay>>> = Arc::new(Mutex::new(Vec::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(relays.clone()))
                .service(start_relay),
        )
        .await;

        let json_string = json!({"nickname": "test_relay"}).to_string();

        let req = test::TestRequest::post()
            .uri("/start_relay")
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let relays_lock = relays.lock().unwrap();
        assert_eq!(relays_lock.len(), 1);
        assert_eq!(relays_lock[0].get_relay_descriptor().nickname, "test_relay");
    }

    #[actix_rt::test]
    async fn test_send_create() {
        let (users, relays) = setup_test_environment(1, 1);
        let user_id = users.lock().unwrap()[0].user_descriptor.id;
        let relay_id = relays.lock().unwrap()[0].get_relay_descriptor().id;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(users.clone()))
                .service(send_create),
        )
        .await;

        let json_string = json!({"relay_id": relay_id}).to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_create", user_id))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_establish_rendezvous() {
        let (users, relays) = setup_test_environment(1, 2);
        let user_id = users.lock().unwrap()[0].user_descriptor.id;
        let relay_id = relays.lock().unwrap()[0].get_relay_descriptor().id;
        let relay_id_2 = relays.lock().unwrap()[1].get_relay_descriptor().id;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(users.clone()))
                .app_data(web::Data::new(relays.clone()))
                .service(send_extend)
                .service(send_create)
                .service(send_establish_rendezvous),
        )
        .await;

        let json_string = json!({"relay_id": relay_id}).to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_create", user_id))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: String = test::read_body_json(resp).await;
        let circuit_id = CircuitId::from_str(&body).expect("Failed to parse CircuitId");

        let json_string = json!({
            "relay_id": relay_id,
            "circuit_id": circuit_id,
            "extend_to": relay_id_2,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_extend", user_id))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let json_string = json!({
            "relay_id": relay_id,
            "circuit_id": circuit_id,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_establish_rendezvous", user_id))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_establish_circuit() {
        let (users, relays) = setup_test_environment(1, 3);
        let user_id = users.lock().unwrap()[0].user_descriptor.id;
        let relay_id_1 = relays.lock().unwrap()[0].get_relay_descriptor().id;
        let relay_id_2 = relays.lock().unwrap()[1].get_relay_descriptor().id;
        let relay_id_3 = relays.lock().unwrap()[2].get_relay_descriptor().id;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(users.clone()))
                .service(establish_circuit),
        )
        .await;

        let json_string = json!({
            "circuit_id": Uuid::new_v4(),
            "relay_address_1": relay_id_1,
            "relay_address_2": relay_id_2,
            "relay_address_3": relay_id_3,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/establish_circuit", user_id))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_establish_introduction() {
        let (users, relays) = setup_test_environment(1, 2);
        let user_id = users.lock().unwrap()[0].user_descriptor.id;
        let relay_id = relays.lock().unwrap()[0].get_relay_descriptor().id;
        let relay_id_2 = relays.lock().unwrap()[1].get_relay_descriptor().id;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(users.clone()))
                .app_data(web::Data::new(relays.clone()))
                .service(send_extend)
                .service(send_create)
                .service(send_establish_introduction),
        )
        .await;

        let json_string = json!({"relay_id": relay_id}).to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_create", user_id))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: String = test::read_body_json(resp).await;
        let circuit_id = CircuitId::from_str(&body).expect("Failed to parse CircuitId");

        let json_string = json!({
            "relay_id": relay_id,
            "circuit_id": circuit_id,
            "extend_to": relay_id_2,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_extend", user_id))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let json_string = json!({
            "relay_id": relay_id,
            "circuit_id": circuit_id,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_establish_introduction", user_id))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_full() {
        let (users, relays) = setup_test_environment(2, 6);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(users.clone()))
                .app_data(web::Data::new(relays.clone()))
                .service(start_relay)
                .service(start_user)
                .service(send_create)
                .service(send_extend)
                .service(establish_circuit)
                .service(send_establish_introduction)
                .service(send_establish_rendezvous)
                .service(send_begin)
                .service(send_introduce1)
                .service(send_rendezvous1)
                .service(send_data_to_relay),
        )
        .await;

        let relay_ids: Vec<Uuid> = relays
            .lock()
            .unwrap()
            .iter()
            .map(|r| r.get_relay_descriptor().id)
            .collect();

        let user_ids: Vec<Uuid> = users
            .lock()
            .unwrap()
            .iter()
            .map(|u| u.user_descriptor.id)
            .collect();

        // User 1 operations
        let circuit_id_1 = Uuid::new_v4();
        let introduction_id = Uuid::new_v4();
        let rendezvous_cookie = Uuid::new_v4();

        // Establish circuit for User 1
        let json_string = json!({
            "circuit_id": circuit_id_1,
            "relay_address_1": relay_ids[0],
            "relay_address_2": relay_ids[1],
            "relay_address_3": relay_ids[2],
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/establish_circuit", user_ids[0]))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Send establish introduction
        let json_string = json!({
            "relay_id": relay_ids[0],
            "circuit_id": circuit_id_1,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!(
                "/users/{}/send_establish_introduction",
                user_ids[0]
            ))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Establish another circuit for User 1
        let circuit_id_2 = Uuid::new_v4();
        let json_string = json!({
            "circuit_id": circuit_id_2,
            "relay_address_1": relay_ids[2],
            "relay_address_2": relay_ids[1],
            "relay_address_3": relay_ids[5],
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/establish_circuit", user_ids[0]))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Send establish rendezvous
        let json_string = json!({
            "relay_id": relay_ids[2],
            "rendezvous_cookie": rendezvous_cookie,
            "circuit_id": circuit_id_2,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_establish_rendezvous", user_ids[1]))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Send rendezvous1
        let json_string = json!({
            "relay_id": relay_ids[2],
            "rendezvous_cookie": rendezvous_cookie,
            "circuit_id": circuit_id_2,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_rendezvous1_to_relay", user_ids[0]))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // User 2 operations
        let circuit_id_3 = Uuid::new_v4();
        let stream_id = Uuid::new_v4();

        // Establish circuit for User 2
        let json_string = json!({
            "circuit_id": circuit_id_3,
            "relay_address_1": relay_ids[3],
            "relay_address_2": relay_ids[4],
            "relay_address_3": relay_ids[5],
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/establish_circuit", user_ids[1]))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Send begin
        let json_string = json!({
            "relay_id": relay_ids[3],
            "circuit_id": circuit_id_3,
            "stream_id": stream_id,
            "begin_relay_id": relay_ids[2],
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_begin_to_relay", user_ids[1]))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Send introduce1
        let introduction_rsa_public = users.lock().unwrap()[0].user_descriptor.rsa_public.clone();
        let json_string = json!({
            "relay_id": relay_ids[3],
            "introduction_id": introduction_id,
            "stream_id": stream_id,
            "rendezvous_point_relay_id": relay_ids[5],
            "rendezvous_cookie": rendezvous_cookie,
            "introduction_rsa_public": introduction_rsa_public,
            "circuit_id": circuit_id_3,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_introduce1_to_relay", user_ids[1]))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Send data
        let data = "Hello, world!".as_bytes().to_vec();
        let json_string = json!({
            "relay_id": relay_ids[3],
            "rendezvous_cookie": rendezvous_cookie,
            "circuit_id": circuit_id_3,
            "data": data,
        })
        .to_string();

        let req = test::TestRequest::post()
            .uri(&format!("/users/{}/send_data_to_relay", user_ids[1]))
            .set_payload(json_string)
            .insert_header(("content-type", "application/json"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
