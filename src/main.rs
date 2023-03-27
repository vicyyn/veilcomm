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

use core::time;
use std::{
    collections::HashMap,
    env,
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
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

pub fn connect_to_peer(node: Node, sender: Sender<Event>) -> Option<Connection> {
    match TcpStream::connect(node.get_addr()) {
        Ok(stream) => {
            println!(
                "[SUCCESS] Connection::establish_connection --> Connected to Peer: {:?}",
                node.get_addr()
            );
            sender
                .send(Event::NewConnection(node, stream.try_clone().unwrap()))
                .unwrap();
            Some(Connection::new(stream, sender.clone()))
        }
        Err(e) => {
            println!(
                "[FAILED] Connection::establish_connection --> Error Connecting to Peer: {}",
                e
            );
            None
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let main_node = Node::new(Ipv4Addr::new(127, 0, 0, 1), args[1].parse().unwrap());
    let mut peers: HashMap<Node, Connection> = HashMap::new();
    let (events_sender, events_receiver) = Event::initialize_channels();
    let mut keys = Keys::new();
    let mut pending: Vec<Node> = vec![];

    listen_for_connections(main_node, events_sender.clone());

    if main_node.port == 8000 {
        let destination = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001);
        connect_to_peer(destination, events_sender.clone());
    }

    loop {
        let event = events_receiver.recv().unwrap();
        match event {
            Event::NewConnection(node, stream) => {
                let connection =
                    Connection::new(stream.try_clone().unwrap(), events_sender.clone());

                if main_node.port == 8001 {
                    let public_key_bytes = keys.dh.public_key().to_vec();
                    let next_node = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8002);
                    let extend_payload = ExtendPayload::new(next_node, &public_key_bytes);
                    let extend_cell = Cell::new_extend_cell(0, extend_payload);

                    connection.write(extend_cell);
                }

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
                    CellCommand::Create => {
                        println!("Received Create!");
                        let create_payload: CreatePayload = cell.payload.into();
                        let aes_key = keys.compute_aes_key(&create_payload.dh_key);
                        keys.add_aes_key(node, aes_key);
                        println!("shared secret (AES) : {:?}", hex::encode(aes_key.get_key()));

                        let public_key_bytes = keys.dh.public_key().to_vec();
                        let cell = Cell::new_created_cell(0, Payload::new(&public_key_bytes));
                        let connection = peers.get(&node).unwrap();
                        connection.write(cell);
                    }
                    CellCommand::Created => {
                        println!("Received Created!");
                        let created_payload: CreatedPayload = cell.payload.into();
                        let aes_key = keys.compute_aes_key(&created_payload.dh_key);
                        keys.add_aes_key(node, aes_key);
                        println!("shared secret (AES) : {:?}", hex::encode(aes_key.get_key()));

                        if !pending.is_empty() {
                            let extended_payload: ExtendedPayload = created_payload.into();
                            let extended_cell = Cell::new_extended_cell(0, extended_payload);
                            let node = pending.pop().unwrap();
                            let connection = peers.get(&node).unwrap();
                            connection.write(extended_cell);
                        }
                    }
                    CellCommand::Extend => {
                        println!("Received Extend Cell!");
                        let extend_payload: ExtendPayload = cell.payload.into();
                        let next_node = extend_payload.get_node();
                        let connection = connect_to_peer(next_node, events_sender.clone());

                        let create_payload: CreatePayload = extend_payload.into();
                        let cell =
                            Cell::new_create_cell(0, Payload::new(&create_payload.serialize()));

                        connection.unwrap().write(cell);
                        pending.push(node);
                    }
                    CellCommand::Extended => {
                        println!("Received Extended Cell!");
                        let extended_payload: ExtendedPayload = cell.payload.into();
                        let aes_key = keys.compute_aes_key(&extended_payload.dh_key);
                        keys.add_aes_key(node, aes_key);
                        println!("shared secret (AES) : {:?}", hex::encode(aes_key.get_key()));
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
