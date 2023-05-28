use crate::RelayDescriptor;
use axum::{extract::Extension, response::IntoResponse, Json};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRelaysOutput {
    pub relays: Vec<RelayDescriptor>,
}

pub async fn get_relays(
    Extension(relay_descriptors): Extension<Arc<RwLock<Vec<RelayDescriptor>>>>,
) -> impl IntoResponse {
    info!("Get Relays Called");
    let relays = relay_descriptors.read().unwrap().iter().cloned().collect();
    Json(GetRelaysOutput { relays })
}
