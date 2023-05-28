use crate::UserDescriptor;
use axum::{extract::Extension, Json};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize, Debug)]
pub struct PublishUserInput {
    pub user_descriptor: UserDescriptor,
}

pub async fn publish_user(
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
