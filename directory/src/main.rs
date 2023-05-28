use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use directory::*;
use log::info;
use std::sync::{Arc, RwLock};

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
    use crate::{
        GetRelaysOutput, GetUsersOutput, PublishRelayInput, PublishUserInput, RelayDescriptor,
        UserDescriptor, DIRECTORY_ADDRESS,
    };
    use reqwest::Client;

    #[tokio::test]
    async fn publish_and_get_users_and_relays() {
        let client = Client::new();
        let url = format!("http://{}", DIRECTORY_ADDRESS);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let res = client
            .post(url.clone() + "/publish_user")
            .json(&PublishUserInput {
                user_descriptor: UserDescriptor::default(),
            })
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let get_users_output = client
            .get(url.clone() + "/get_users")
            .send()
            .await
            .unwrap()
            .json::<GetUsersOutput>()
            .await
            .unwrap();
        assert_eq!(get_users_output.users.len(), 1);
        let res = client
            .post(url.clone() + "/publish_relay")
            .json(&PublishRelayInput {
                relay_descriptor: RelayDescriptor::default(),
            })
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let get_relays_output = client
            .get(url.clone() + "/get_relays")
            .send()
            .await
            .unwrap()
            .json::<GetRelaysOutput>()
            .await
            .unwrap();
        assert_eq!(get_relays_output.relays.len(), 1);
    }
}
