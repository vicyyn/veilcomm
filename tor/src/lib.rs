pub mod aes_key;
pub mod circuit;
pub mod circuit_node;
pub mod circuits;
pub mod connections;
pub mod constants;
pub mod directory;
pub mod keys;
pub mod network;
pub mod pending_response;
pub mod pending_responses;
pub mod streams;
// pub mod tor_event;
pub mod tor_state;

pub use aes_key::*;
pub use circuit::*;
pub use circuit_node::*;
pub use circuits::*;
pub use connections::*;
pub use constants::*;
pub use directory::*;
pub use keys::*;
pub use network::*;
pub use pending_response::*;
pub use pending_responses::*;
pub use streams::*;
// pub use tor_event::*;
pub use tor_state::*;

use std::{io::Error, net::TcpStream, sync::mpsc::Sender};

pub async fn connect_to_peer(
    node: Node,
    connection_event_sender: Sender<ConnectionEvent>,
) -> Result<(), Error> {
    match TcpStream::connect(node.get_addr()) {
        Ok(stream) => {
            println!(
                "[SUCCESS] tor::connect_to_peer --> Connected to Peer: {:?}",
                node.get_addr()
            );
            connection_event_sender
                .send(ConnectionEvent::NewConnection(
                    node,
                    stream.try_clone().unwrap(),
                ))
                .unwrap();
            Ok(())
        }
        Err(e) => {
            println!(
                "[FAILED] tor::connect_to_peer --> Error Connecting to Peer: {}",
                e
            );
            Err(e)
        }
    }
}
