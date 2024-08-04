use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::{error, info};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use veilcomm2::api_address;
use veilcomm2::relay::{Relay, RelayDescriptor};
use veilcomm2::user::{User, UserDescriptor};

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
        let address = api_address();
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
                    .service(fetch_relays)
                    .service(establish_circuit)
                    .service(send_establish_rendezvous_to_relay)
                    .service(send_begin_to_relay)
                    .service(send_introduce1_to_relay)
                    .service(send_data_to_relay)
                    .service(send_establish_introduction_to_relay)
                    .service(add_introduction_point)
                    .service(update_introduction_points)
                    .service(send_rendezvous1_to_relay)
                    .service(get_relays)
                    .service(get_users)
                // .service(get_relay_logs)
                // .service(get_user_logs)
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

#[derive(Deserialize)]
pub struct StartRelayBody {
    pub nickname: String,
}

#[post("/start_relay")]
async fn start_relay(
    data: web::Data<Arc<Mutex<Vec<Relay>>>>,
    body: web::Json<StartRelayBody>,
) -> impl Responder {
    info!("Starting relay: {:?}", body.nickname);
    let mut relays = data.lock().await;
    // generate random address that can be used
    let address = generate_random_socket_address();
    let relay = Relay::new(address, body.nickname.clone());
    relay.start().await.unwrap();
    relays.push(relay);
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct StartUserBody {
    pub nickname: String,
}

#[post("/start_user")]
async fn start_user(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    body: web::Json<StartUserBody>,
) -> impl Responder {
    info!("Starting user: {:?}", body.nickname);
    let mut users = data.lock().await;
    let user = User::new(body.nickname.clone());
    user.start().await.unwrap();
    users.push(user);
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct SendExtendBody {
    pub circuit_id: Uuid,
    pub relay_socket: SocketAddr,
    pub extend_to: SocketAddr,
}

#[post("/users/{user_id}/send_extend_to_relay")]
async fn send_extend_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<SendExtendBody>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_extend_to_relay(body.relay_socket, body.extend_to, body.circuit_id)
        .await
        .unwrap();
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct SendCreateBody {
    pub relay_socket: SocketAddr,
}

#[derive(Serialize)]
pub struct RelayLog {
    pub nickname: String,
    pub logs: Vec<String>,
}

// fetch logs from relay
// #[get("/relay_logs")]
// async fn get_relay_logs(data: web::Data<Arc<Mutex<Vec<Relay>>>>) -> impl Responder {
//     let data_lock = data.lock().await;
//     let mut all_logs = vec![];
//     for relay in data_lock.iter() {
//         let logs = relay.get_logs().await;
//         all_logs.push(RelayLog {
//             nickname: relay.get_relay_descriptor().nickname,
//             logs,
//         });
//     }
//     HttpResponse::Ok().json(all_logs)
// }

#[derive(Serialize)]
pub struct UserLog {
    pub nickname: String,
    pub logs: Vec<String>,
}

// fetch logs from user
// #[get("/user_logs")]
// async fn get_user_logs(data: web::Data<Arc<Mutex<Vec<User>>>>) -> impl Responder {
//     let data_lock = data.lock().await;
//     let mut all_logs = vec![];
//     for user in data_lock.iter() {
//         let logs = user.get_logs().await;
//         all_logs.push(UserLog {
//             nickname: user.user_descriptor.nickname.clone(),
//             logs,
//         });
//     }
//     HttpResponse::Ok().json(all_logs)
// }

// call user.send_create_to_relay endpoint
#[post("/users/{user_id}/send_create_to_relay")]
async fn send_create_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<SendCreateBody>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let circuit_id = Uuid::new_v4();
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_create_to_relay(body.relay_socket, circuit_id)
        .await
        .unwrap();
    HttpResponse::Ok().json(circuit_id.to_string())
}

#[get("/relays")]
async fn get_relays(data: web::Data<Arc<Mutex<Vec<Relay>>>>) -> impl Responder {
    info!("Fetching all relays");
    let relays = data.lock().await;
    let relays_descriptors = relays
        .iter()
        .map(|r| r.get_relay_descriptor())
        .collect::<Vec<RelayDescriptor>>();
    HttpResponse::Ok().json(relays_descriptors)
}

#[get("/users")]
async fn get_users(data: web::Data<Arc<Mutex<Vec<User>>>>) -> impl Responder {
    info!("Fetching all users");
    let users = data.lock().await;
    let users_descriptors = users
        .iter()
        .map(|u| u.user_descriptor.clone())
        .collect::<Vec<UserDescriptor>>();
    HttpResponse::Ok().json(users_descriptors)
}

#[derive(Deserialize)]
pub struct FetchRelaysBody {}

#[post("/users/{user_id}/fetch_relays")]
async fn fetch_relays(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    // fetch relays for user, if error respond with error
    user.fetch_relays().await.unwrap();
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct EstablishCircuitBody {
    pub circuit_id: Uuid,
    pub relay_address_1: SocketAddr,
    pub relay_address_2: SocketAddr,
    pub relay_address_3: SocketAddr,
}

#[post("/users/{user_id}/establish_circuit")]
async fn establish_circuit(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<EstablishCircuitBody>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.establish_circuit(
        body.circuit_id,
        body.relay_address_1,
        body.relay_address_2,
        body.relay_address_3,
    )
    .await
    .unwrap();
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct SendEstablishRendezvousBody {
    pub relay_socket: SocketAddr,
    pub circuit_id: Uuid,
}

#[post("/users/{user_id}/send_establish_rendezvous_to_relay")]
async fn send_establish_rendezvous_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<SendEstablishRendezvousBody>,
) -> impl Responder {
    println!("send_establish_rendezvous_to_relay");
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    let rendezvous_cookie = Uuid::new_v4();
    user.send_establish_rendezvous_to_relay(body.relay_socket, rendezvous_cookie, body.circuit_id)
        .await
        .unwrap();
    HttpResponse::Ok().json(rendezvous_cookie.to_string())
}

#[derive(Deserialize)]
pub struct SendBeginToRelayBody {
    pub relay_socket: SocketAddr,
    pub circuit_id: Uuid,
    pub begin_relay_socket: SocketAddr,
}

#[post("/users/{user_id}/send_begin_to_relay")]
async fn send_begin_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<SendBeginToRelayBody>,
) -> impl Responder {
    println!("send_begin_to_relay");
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    let stream_id = Uuid::new_v4();
    user.send_begin_to_relay(
        body.relay_socket,
        body.circuit_id,
        stream_id,
        body.begin_relay_socket,
    )
    .await
    .unwrap();
    HttpResponse::Ok().json(stream_id.to_string())
}

#[derive(Deserialize)]
pub struct SendIntroduce1Body {
    pub relay_socket: SocketAddr,
    pub introduction_id: Uuid,
    pub stream_id: Uuid,
    pub rendezvous_point_socket: SocketAddr,
    pub rendezvous_cookie: Uuid,
    pub introduction_rsa_public: Vec<u8>,
    pub circuit_id: Uuid,
}

#[post("/users/{user_id}/send_introduce1_to_relay")]
async fn send_introduce1_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<SendIntroduce1Body>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_introduce1_to_relay(
        body.relay_socket,
        body.introduction_id,
        body.stream_id,
        body.rendezvous_point_socket,
        body.rendezvous_cookie,
        body.introduction_rsa_public.clone(),
        body.circuit_id,
    )
    .await
    .unwrap();
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct SendDataToRelayBody {
    pub relay_socket: SocketAddr,
    pub rendezvous_cookie: Uuid,
    pub circuit_id: Uuid,
    pub data: Vec<u8>,
}

#[post("/users/{user_id}/send_data_to_relay")]
async fn send_data_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<SendDataToRelayBody>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_data_to_relay(
        body.relay_socket,
        body.rendezvous_cookie,
        body.circuit_id,
        body.data.clone(),
    )
    .await
    .unwrap();
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct SendEstablishIntroductionBody {
    pub relay_socket: SocketAddr,
    pub circuit_id: Uuid,
}

#[post("/users/{user_id}/send_establish_introduction_to_relay")]
async fn send_establish_introduction_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<SendEstablishIntroductionBody>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    let introduction_id = Uuid::new_v4();
    user.send_establish_introduction_to_relay(body.relay_socket, introduction_id, body.circuit_id)
        .await
        .unwrap();
    HttpResponse::Ok().json(introduction_id.to_string())
}

#[derive(Deserialize)]
pub struct AddIntroductionPointBody {
    pub introduction_id: Uuid,
    pub relay_address: SocketAddr,
}

#[post("/users/{user_id}/add_introduction_point")]
async fn add_introduction_point(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<AddIntroductionPointBody>,
) -> impl Responder {
    let mut data_lock = data.lock().await;
    let user = data_lock
        .iter_mut()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.add_introduction_point(body.introduction_id, body.relay_address);
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct UpdateIntroductionPointsBody {}

#[post("/users/{user_id}/update_introduction_points")]
async fn update_introduction_points(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.update_introduction_points().await.unwrap();
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct SendRendezvous1Body {
    pub relay_socket: SocketAddr,
    pub rendezvous_cookie: Uuid,
    pub circuit_id: Uuid,
}

#[post("/users/{user_id}/send_rendezvous1_to_relay")]
async fn send_rendezvous1_to_relay(
    data: web::Data<Arc<Mutex<Vec<User>>>>,
    user_id: web::Path<Uuid>,
    body: web::Json<SendRendezvous1Body>,
) -> impl Responder {
    let data_lock = data.lock().await;
    let user = data_lock
        .iter()
        .find(|u| u.user_descriptor.id == *user_id)
        .unwrap();
    user.send_rendezvous1_to_relay(body.relay_socket, body.rendezvous_cookie, body.circuit_id)
        .await
        .unwrap();
    HttpResponse::Ok().finish()
}

pub fn generate_random_socket_address() -> SocketAddr {
    let mut rng = rand::thread_rng();

    // Generate a random IPv4 address
    let ip = Ipv4Addr::LOCALHOST;
    // Generate a random port number (avoiding well-known ports)
    let port = rng.gen_range(1024..65535);

    SocketAddr::new(IpAddr::V4(ip), port)
}
