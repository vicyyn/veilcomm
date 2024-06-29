mod data;
mod directory;
mod relay;
mod user;
mod utils;

use data::*;
use directory::Directory;
use relay::Relay;
use user::User;

use simple_logger::SimpleLogger;
use std::{net::SocketAddr, str::FromStr};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    SimpleLogger::with_level(SimpleLogger::new(), log::LevelFilter::Info)
        .init()
        .unwrap();

    let directory_address = SocketAddr::from_str("127.0.0.1:3030").unwrap();
    let directory = Directory::new(directory_address);
    directory.start();

    let relay_address = SocketAddr::from_str("127.0.0.1:3031").unwrap();
    let relay = Relay::new(relay_address, "Relay1".to_string());
    let relay_descriptor = relay.get_relay_descriptor();

    tokio::spawn(async move {
        relay.start(directory_address).await.unwrap();
    });

    let relay_address_2 = SocketAddr::from_str("127.0.0.1:3032").unwrap();
    let relay_2 = Relay::new(relay_address_2, "Relay2".to_string());

    tokio::spawn(async move {
        relay_2.start(directory_address).await.unwrap();
    });

    let relay_address_3 = SocketAddr::from_str("127.0.0.1:3033").unwrap();
    let relay_3 = Relay::new(relay_address_3, "Relay3".to_string());

    tokio::spawn(async move {
        relay_3.start(directory_address).await.unwrap();
    });

    let user = User::new("User".to_string(), vec![7, 8, 9]);

    tokio::spawn(async move {
        user.start(directory_address).await.unwrap();
        user.fetch_relays(directory_address).await.unwrap();
        user.connect_to_relay(relay_descriptor).await.unwrap();
        let circuit_id = Uuid::new_v4();
        user.send_create_to_relay(relay_address, circuit_id)
            .await
            .unwrap();
        user.listen_for_event(Event(PayloadType::Created, relay_address))
            .await
            .unwrap();
        user.send_extend_to_relay(relay_address, relay_address_2, circuit_id)
            .await
            .unwrap();
        user.listen_for_event(Event(PayloadType::Extended, relay_address))
            .await
            .unwrap();
        user.send_extend_to_relay(relay_address, relay_address_3, circuit_id)
            .await
            .unwrap();
        user.listen_for_event(Event(PayloadType::Extended, relay_address))
            .await
    });

    loop {}
}
