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
pub mod tor_event;
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
pub use tor_event::*;
pub use tor_state::*;

use std::{
    io::Error,
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
    thread,
};

pub fn connect(node: Node) -> Result<TcpStream, Error> {
    match TcpStream::connect(node.get_addr()) {
        Ok(stream) => {
            println!(
                "[SUCCESS] tor::connect_to_peer --> Connected to Peer: {:?}",
                node.get_addr()
            );
            Ok(stream)
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

pub fn listen_for_connections(node: Node, sender: Sender<TorEvent>) {
    thread::spawn(move || loop {
        let socket = TcpListener::bind(node.get_addr())
            .expect("[FAILED] tor::listen_for_connections --> Error while binding TcpSocket to specified addr");

        match socket.accept() {
            Ok((stream, addr)) => {
                println!(
                    "[SUCCESS] tor::listen_for_connections - New client connected: {:?}",
                    addr
                );
                sender
                    .send(TorEvent::NewPeerConnection(addr.into(), stream))
                    .unwrap()
            }
            Err(e) => {
                println!(
                    "[FAILED] tor::listen_for_connections - Error accepting client connection: {}",
                    e
                );
            }
        }
    });
}
