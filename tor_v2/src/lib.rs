pub mod cli;
pub mod crypto;
pub mod data;
pub mod events;
pub mod network;
pub mod routing;

pub use cli::*;
pub use crypto::*;
pub use data::*;
pub use events::*;
pub use network::*;
pub use routing::*;

use directory::RelayDescriptor;
use events::tor_change::TorChange;
use log::info;
use std::{
    net::SocketAddrV4,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    thread, time,
};

pub fn start_peer(socket_address: SocketAddrV4) -> (Sender<TorEvent>, Receiver<TorChange>) {
    println!("Starting Peer...");
    let circuits = Circuits::new();
    let streams = Streams::new();
    let connections = Connections::new();
    let pending_responses = PendingResponses::new();
    let keys = Arc::new(RwLock::new(Keys::new()));
    let relays = RelayDescriptors::new();
    let user_descriptors = UserDescriptors::new();
    let cookies = Cookies::new();
    let introduction_points = IntroductionPoints::new();
    let (tor_event_sender, tor_event_receiver) = channel();
    let (tor_change_sender, tor_change_receiver) = channel();
    let circ_ids = CircIds::new();
    let users = Users::new();

    let relay = RelayDescriptor::new(
        "Joe".to_string(),
        keys.read()
            .unwrap()
            .relay_id_rsa
            .public_key_to_der()
            .unwrap(),
        socket_address,
        "joe@gmail.com".to_string(),
    );

    let user_descriptor = Arc::new(RwLock::new(keys.read().unwrap().get_user_descriptor()));

    publish_relay(relay);

    listen_for_connections(socket_address, tor_event_sender.clone());
    std::thread::spawn({
        let tor_event_sender = tor_event_sender.clone();
        let tor_change_sender = tor_change_sender.clone();

        move || loop {
            let tor_event = tor_event_receiver.recv().unwrap();
            process_tor_event(
                tor_event,
                connections.clone(),
                pending_responses.clone(),
                tor_event_sender.clone(),
                tor_change_sender.clone(),
                Arc::clone(&keys),
                circuits.clone(),
                relays.clone(),
                user_descriptors.clone(),
                streams.clone(),
                Arc::clone(&user_descriptor),
                cookies.clone(),
                introduction_points.clone(),
                circ_ids.clone(),
                users.clone(),
            );
        }
    });

    return (tor_event_sender.clone(), tor_change_receiver);
}
