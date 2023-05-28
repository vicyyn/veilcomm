use crate::UserDescriptor;
use axum::{extract::Extension, response::IntoResponse, Json};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUsersOutput {
    pub users: Vec<UserDescriptor>,
}

pub async fn get_users(
    Extension(user_descriptors): Extension<Arc<RwLock<Vec<UserDescriptor>>>>,
) -> impl IntoResponse {
    info!("Get Users Called");
    let users = user_descriptors.read().unwrap().iter().cloned().collect();
    Json(GetUsersOutput { users })
}
