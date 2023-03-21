pub mod aes_key;
pub mod cell;
pub mod cell_command;
pub mod circuit;
pub mod connection;
pub mod directory;
pub mod event;
pub mod key;
pub mod keys;
pub mod node;
pub mod payload;
pub mod payloads;
pub mod relay_command;
pub mod utils;

pub use aes_key::*;
pub use cell::*;
pub use cell_command::*;
pub use circuit::*;
pub use connection::*;
pub use directory::*;
pub use event::*;
pub use key::*;
pub use keys::*;
pub use node::*;
pub use payload::*;
pub use payloads::*;
pub use relay_command::*;
pub use utils::*;

use std::{
    collections::HashMap,
    env,
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender, SyncSender},
    thread,
};

pub fn listen_for_connections(node: Node, sender: Sender<Event>) {
    thread::spawn(move || loop {
        let socket = TcpListener::bind(node.get_addr())
            .expect("[FAILED] Connection::new --> Error while binding TcpSocket to specified addr");

        match socket.accept() {
            Ok((stream, addr)) => {
                println!(
                    "[SUCCESS] main::listen_for_connections - New client connected: {:?}",
                    addr
                );
                sender
                    .send(Event::NewConnection(addr.into(), stream))
                    .unwrap()
            }
            Err(e) => {
                println!(
                    "[FAILED] main::listen_for_connections - Error accepting client connection: {}",
                    e
                );
            }
        }
    });
}

pub fn connect_to_peer(node: Node, sender: Sender<Event>) {
    match TcpStream::connect(node.get_addr()) {
        Ok(stream) => {
            println!(
                "[SUCCESS] Connection::establish_connection --> Connected to Peer: {:?}",
                node.get_addr()
            );
            sender.send(Event::NewConnection(node, stream)).unwrap()
        }
        Err(e) => {
            println!(
                "[FAILED] Connection::establish_connection --> Error Connecting to Peer: {}",
                e
            );
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let node = Node::new(Ipv4Addr::new(127, 0, 0, 1), args[1].parse().unwrap());
    let mut peers: HashMap<Node, Connection> = HashMap::new();
    let (events_sender, events_receiver) = Event::initialize_channels();

    let directory = Directory::new();
    let bootstrap_nodes = directory.get_bootstrap_nodes();
    for destination in bootstrap_nodes {
        connect_to_peer(destination, events_sender.clone());
    }

    listen_for_connections(node, events_sender.clone());

    loop {
        let event = events_receiver.recv().unwrap();
        match event {
            Event::NewConnection(node, stream) => {
                let connection =
                    Connection::new(stream.try_clone().unwrap(), events_sender.clone());
                connection.write(Cell::new_ping_cell());
                peers.insert(node, connection);
            }
            Event::ReceiveCell(node, cell) => match CellCommand::try_from(cell.command) {
                Ok(command) => match command {
                    CellCommand::Ping => {
                        println!("Received Ping!");
                        let connection = peers.get(&node).unwrap();
                        connection.write(Cell::new_pong_cell());
                    }
                    CellCommand::Pong => {
                        println!("Received Pong!");
                    }
                    _ => println!("Other"),
                },
                Err(e) => println!(
                    "[FAILED] Connection::handle_cell --> Error getting cell command: {}",
                    e
                ),
            },
            _ => {}
        }
    }
}
