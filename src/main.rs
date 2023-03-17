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
    io::Write,
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

#[derive(Debug)]
pub enum Event {
    NewConnection(TcpStream, SocketAddr),
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
                sender.send(Event::NewConnection(stream, addr)).unwrap()
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

pub fn listen_for_peer(peer: Peer, sender: Sender<Event>) {
    thread::spawn(move || loop {
        let cell = peer.connection_channels.read_receiver.recv().unwrap();
        sender.send(Event::ReceiveCell(peer.node, cell)).unwrap();
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let node = Node::new(Ipv4Addr::new(127, 0, 0, 1), args[1].parse().unwrap());
    let (events_sender, events_receiver): (Sender<Event>, Receiver<Event>) = mpsc::channel();

    let mut peers: HashMap<Id, Peer> = HashMap::new();

    let destination = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001);
    match TcpStream::connect(destination.get_addr()) {
        Ok(mut stream) => {
            println!(
                "[SUCCESS] Connection::establish_connection --> Connected to Peer: {:?}",
                destination.get_addr()
            );
            let peer = Peer::new(destination, stream.try_clone().unwrap());
            peers.insert(destination.id, peer);

            let cell = Cell::default();
            stream.write(&cell.serialize()).unwrap();
        }
        Err(e) => {
            println!(
                "[FAILED] Connection::establish_connection --> Error Connecting to Peer: {}",
                e
            );
        }
    }

    listen_for_connections(node, events_sender.clone());
    loop {
        let event = events_receiver.recv().unwrap();
        match event {
            Event::NewConnection(stream, addr) => {
                let peer = Peer::new(addr.into(), stream);
                // peers.insert(peer.node.id, peer);
                listen_for_peer(peer, events_sender.clone());
            }
            Event::ReceiveCell(node, cell) => {
                println!("{:?} , {:?}", node, cell);
            }
            _ => {}
        }
    }
}
