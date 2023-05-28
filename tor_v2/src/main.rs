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

use clap::Parser;
use directory::RelayDescriptor;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock,
    },
};

fn main() {
    env_logger::init();
    let args = Args::parse();
    start_peer(SocketAddrV4::new(Ipv4Addr::LOCALHOST, args.port));
    loop {}
}

fn start_peer(socket_address: SocketAddrV4) -> Sender<TorEvent> {
    let circuits = Circuits::new();
    let streams = Streams::new();
    let connections = Connections::new();
    let pending_responses = PendingResponses::new();
    let keys = Arc::new(RwLock::new(Keys::new()));
    let relays = RelayDescriptors::new();
    let user_descriptors = UserDescriptors::new();
    let cookies = Cookies::new();
    let introduction_points = IntroductionPoints::new();
    let (connection_events_sender, connection_events_receiver) = channel();
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

    listen_for_connections(socket_address, connection_events_sender.clone());
    std::thread::spawn({
        let connection_events_sender = connection_events_sender.clone();
        move || loop {
            let connection_event = connection_events_receiver.recv().unwrap();
            process_tor_event(
                connection_event,
                connections.clone(),
                pending_responses.clone(),
                connection_events_sender.clone(),
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

    return connection_events_sender.clone();
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::thread;

    use super::*;

    #[test]
    fn test_tor() {
        let node1 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8001);
        let node2 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8002);
        let node3 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8003);
        let node4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8004);
        let node5 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8005);
        let node6 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8006);
        let node7 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8007);
        let node8 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8008);

        let node9 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8009);
        let node10 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8010);
        let node11 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8011);

        _ = start_peer(node11);
        _ = start_peer(node10);
        _ = start_peer(node9);

        let t8 = start_peer(node8);
        let _ = start_peer(node7);
        let _ = start_peer(node6);
        let _ = start_peer(node5);
        let _ = start_peer(node4);
        let _ = start_peer(node3);
        let _ = start_peer(node2);
        let t1 = start_peer(node1);

        println!(" First Circuit * * * * * * * * * *");
        create_circuit(0, t1.clone(), node2, node3, node4);

        println!(" Second Circuit * * * * * * * * * *");
        create_circuit(0, t8.clone(), node7, node6, node5);
        println!(" * * * * * * * * * *");

        println!(" - - - - - - -");
        t8.send(TorEvent::EstablishIntro(0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t8.send(TorEvent::PublishUserDescriptor).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        t1.send(TorEvent::EstablishRendPoint(0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(TorEvent::OpenStream(0, node5, 0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(TorEvent::FetchFromDirectory).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(TorEvent::Introduce1(0)).unwrap();
        thread::sleep(time::Duration::from_millis(30000));

        println!(" - - - - - - -");
        let relay_payload = RelayPayload::new_data_payload("Hello!".as_bytes(), 3);
        let cell = Cell::new_relay_cell(0, relay_payload);
        t1.send(TorEvent::SendCell(cell)).unwrap();

        loop {}
    }
}
