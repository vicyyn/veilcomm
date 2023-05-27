use axum::{
    extract::Extension,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

pub mod directory_event;
pub mod relay_descriptor;
pub mod user_descriptor;

pub use directory_event::*;
pub use relay_descriptor::*;
pub use user_descriptor::*;

pub const DIRECTORY_ADDRESS: &'static str = "127.0.0.1:8100";

#[derive(Deserialize, Serialize, Debug)]
struct PublishUserInput {
    pub user_descriptor: UserDescriptor,
}

async fn publish_user(
    user_descriptors: Extension<Arc<RwLock<Vec<UserDescriptor>>>>,
    Json(publish_user_input): Json<PublishUserInput>,
) {
    info!(
        "Publish User Called, {:?}",
        hex::encode(&publish_user_input.user_descriptor.address)
    );
    user_descriptors
        .write()
        .unwrap()
        .push(publish_user_input.user_descriptor);
}

#[derive(Debug, Serialize, Deserialize)]
struct GetUsersOutput {
    users: Vec<UserDescriptor>,
}

async fn get_users(
    Extension(user_descriptors): Extension<Arc<RwLock<Vec<UserDescriptor>>>>,
) -> impl IntoResponse {
    info!("Get Users Called");
    let users = user_descriptors.read().unwrap().iter().cloned().collect();
    Json(GetUsersOutput { users })
}

#[derive(Deserialize, Serialize, Debug)]
struct PublishRelayInput {
    pub relay_descriptor: RelayDescriptor,
}

async fn publish_relay(
    relay_descriptors: Extension<Arc<RwLock<Vec<RelayDescriptor>>>>,
    Json(publish_relay_input): Json<PublishRelayInput>,
) -> &'static str {
    info!(
        "Publish Relay Called , {:?}",
        publish_relay_input.relay_descriptor.socket_address
    );
    relay_descriptors
        .write()
        .unwrap()
        .push(publish_relay_input.relay_descriptor);

    "Relay Descriptor Added"
}

#[derive(Debug, Serialize, Deserialize)]
struct GetRelaysOutput {
    relays: Vec<RelayDescriptor>,
}

async fn get_relays(
    Extension(relay_descriptors): Extension<Arc<RwLock<Vec<RelayDescriptor>>>>,
) -> impl IntoResponse {
    info!("Get Relays Called");
    let relays = relay_descriptors.read().unwrap().iter().cloned().collect();
    Json(GetRelaysOutput { relays })
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let relay_descriptors: Arc<RwLock<Vec<RelayDescriptor>>> = Arc::new(RwLock::new(Vec::new()));
    let user_descriptors: Arc<RwLock<Vec<UserDescriptor>>> = Arc::new(RwLock::new(Vec::new()));

    let app = Router::new()
        .route("/get_users", get(get_users))
        .route("/get_relays", get(get_relays))
        .route("/publish_user", post(publish_user))
        .route("/publish_relay", post(publish_relay))
        .layer(Extension(user_descriptors))
        .layer(Extension(relay_descriptors));

    info!("Directory Started listening on {}", DIRECTORY_ADDRESS);
    axum::Server::bind(&DIRECTORY_ADDRESS.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::{
        GetRelaysOutput, GetUsersOutput, PublishRelayInput, PublishUserInput, RelayDescriptor,
        UserDescriptor, DIRECTORY_ADDRESS,
    };

    #[tokio::test]
    async fn publish_and_get_users_and_relays() {
        let client = Client::new();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let res = client
            .post("http://".to_string() + DIRECTORY_ADDRESS + "/publish_user")
            .json(&PublishUserInput {
                user_descriptor: UserDescriptor::default(),
            })
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let get_users_output = client
            .get("http://".to_string() + DIRECTORY_ADDRESS + "/get_users")
            .send()
            .await
            .unwrap()
            .json::<GetUsersOutput>()
            .await
            .unwrap();
        assert_eq!(get_users_output.users.len(), 1);
        let res = client
            .post("http://".to_string() + DIRECTORY_ADDRESS + "/publish_relay")
            .json(&PublishRelayInput {
                relay_descriptor: RelayDescriptor::default(),
            })
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let get_relays_output = client
            .get("http://".to_string() + DIRECTORY_ADDRESS + "/get_relays")
            .send()
            .await
            .unwrap()
            .json::<GetRelaysOutput>()
            .await
            .unwrap();
        assert_eq!(get_relays_output.relays.len(), 1);
    }
}
