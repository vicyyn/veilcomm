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
    net::{Ipv4Addr, TcpListener},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub fn listen_for_connections(node: Node, sender: Sender<i32>) {
    thread::spawn(move || loop {
        let socket = TcpListener::bind(node.get_addr())
            .expect("[FAILED] Connection::new --> Error while binding TcpSocket to specified addr");

        match socket.accept() {
            Ok((stream, addr)) => {
                println!("[SUCCESS] New client connected: {:?}", addr);
                sender.send(3).unwrap()
            }
            Err(e) => {
                println!("[FAILED] Error accepting client connection: {}", e);
            }
        }
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let node = Node::new(Ipv4Addr::new(127, 0, 0, 1), args[1].parse().unwrap());
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    listen_for_connections(node, tx);
    let peers: HashMap<Id, Peer> = HashMap::new();

    let x = rx.recv().unwrap();
}
