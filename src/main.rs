pub mod aes_key;
pub mod cell;
pub mod cell_command;
pub mod circuit;
pub mod connection2;
pub mod key;
pub mod keys;
pub mod node;
pub mod payload;
pub mod payloads;
pub mod peer;
pub mod relay_command;
pub mod utils;

pub use aes_key::*;
pub use cell::*;
pub use cell_command::*;
pub use circuit::*;
pub use connection2::*;
pub use key::*;
pub use keys::*;
pub use node::*;
pub use payload::*;
pub use payloads::*;
pub use peer::*;
pub use relay_command::*;
pub use utils::*;

use std::{
    collections::HashMap,
    env,
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

#[derive(Debug)]
pub enum Event {
    NewConnection(Node, TcpStream),
    ReceiveCell(Node, Cell),
}

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

pub fn listen_peer(node: Node, receiver: Receiver<Cell>, sender: Sender<Event>) {
    thread::spawn(move || loop {
        let cell = receiver.recv().unwrap();
        sender.send(Event::ReceiveCell(node, cell)).unwrap();
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let node = Node::new(Ipv4Addr::new(127, 0, 0, 1), args[1].parse().unwrap());
    let (events_sender, events_receiver): (Sender<Event>, Receiver<Event>) = mpsc::channel();

    let mut peers: HashMap<Node, Sender<Cell>> = HashMap::new();
    let destination = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8000);
    connect_to_peer(destination, events_sender.clone());
    listen_for_connections(node, events_sender.clone());

    loop {
        let event = events_receiver.recv().unwrap();
        match event {
            Event::NewConnection(node, stream) => {
                let connections_channel = Connection::open(stream.try_clone().unwrap());
                peers.insert(node, connections_channel.write_sender.clone());

                listen_peer(
                    node,
                    connections_channel.read_receiver,
                    events_sender.clone(),
                );
                connections_channel
                    .write_sender
                    .send(Cell::default())
                    .unwrap();
            }
            Event::ReceiveCell(node, cell) => {
                println!("{:?} , {:?}", node, cell);
            }
            _ => {}
        }
    }
}
