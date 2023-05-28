use crate::RelayDescriptor;
use axum::{extract::Extension, Json};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize, Debug)]
pub struct PublishRelayInput {
    pub relay_descriptor: RelayDescriptor,
}

pub async fn publish_relay(
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
