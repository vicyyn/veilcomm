mod directory;
mod payloads;
mod relay;
mod user;
mod utils;

use directory::Directory;
use payloads::{Event, PayloadType};
use relay::Relay;
use user::User;

use simple_logger::SimpleLogger;
use std::{net::SocketAddr, str::FromStr};

#[tokio::main]
async fn main() {
    SimpleLogger::with_level(SimpleLogger::new(), log::LevelFilter::Info)
        .init()
        .unwrap();

    let directory_address = SocketAddr::from_str("127.0.0.1:3030").unwrap();
    let directory = Directory::new(directory_address);
    directory.start();

    let relay_address = SocketAddr::from_str("127.0.0.1:3031").unwrap();
    let relay = Relay::new(relay_address, "John".to_string());

    tokio::spawn(async move {
        relay.start(directory_address).await.unwrap();
    });

    let relay_address_2 = SocketAddr::from_str("127.0.0.1:3032").unwrap();
    let relay_2 = Relay::new(relay_address_2, "Mark".to_string());

    tokio::spawn(async move {
        relay_2.start(directory_address).await.unwrap();
    });

    let user = User::new(
        vec![7, 8, 9],
        vec![SocketAddr::from_str("127.0.0.1:3031").unwrap()],
    );

    tokio::spawn(async move {
        user.start(directory_address).await.unwrap();
        user.fetch_relays(directory_address).await.unwrap();
        user.connect_to_relay(relay_address).await.unwrap();
        user.send_create_to_relay(relay_address).await.unwrap();
        user.listen_for_event(Event(PayloadType::Created, relay_address))
            .await
            .unwrap();
        user.send_extend_to_relay(relay_address, relay_address_2)
            .await
            .unwrap();
        user.listen_for_event(Event(PayloadType::Extended, relay_address))
            .await
            .unwrap();
    });

    loop {}
}
