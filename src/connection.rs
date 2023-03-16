use openssl::bn::BigNum;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

pub struct Connection {
    pub node: Node,
    pub keys: Keys,
    pub tcp_streams: Arc<RwLock<HashMap<Node, TcpStream>>>,
}

impl Connection {
    pub fn new(node: Node) -> Self {
        Self {
            node,
            keys: Keys::new(),
            tcp_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_stream(&mut self, node: Node, stream: TcpStream) {
        self.tcp_streams.write().unwrap().insert(node, stream);
    }

    pub fn add_aes_key(&mut self, node: Node, aes_key: AESKey) {
        self.keys.aes_keys.insert(node, aes_key);
    }

    pub fn open_receiver(self: Arc<Self>, receiver: Receiver<Cell>) {
        thread::spawn(move || loop {
            match receiver.recv() {
                Ok(cell) => {
                    println!(
                        "[SUCCESS] Connection::listen_channel --> Received Cell: {:?}",
                        cell
                    );
                }
                Err(e) => {
                    println!(
                        "[FAILED] Connection::listen_channel --> Error while Receiving Cell: {:?}",
                        e
                    );
                }
            };
        });
    }

    pub fn open(self: Arc<Self>, node: Node) {
        thread::spawn(move || loop {
            let socket = TcpListener::bind(node.get_addr()).expect(
                "[FAILED] Connection::new --> Error while binding TcpSocket to specified addr",
            );
            println!(
                "[SUCCESS] Connection::open --> Listening for incoming connections: {:?}",
                node.get_addr()
            );

            loop {
                match socket.accept() {
                    Ok((mut stream, addr)) => {
                        println!(
                            "[SUCCESS] Connection::open --> New client connected: {:?}",
                            addr
                        );

                        let node = stream.local_addr().unwrap();
                        Arc::clone(&self).receive(stream);
                    }
                    Err(e) => {
                        println!(
                            "[FAILED] Connection::open --> Error accepting client connection: {}",
                            e
                        );
                    }
                }
            }
        });
    }

    pub fn receive(self: Arc<Self>, stream: TcpStream) {
        let mut buffer = [0u8; CELL_SIZE];
        let mut stream_clone = stream.try_clone().unwrap();

        thread::spawn(move || loop {
            match stream_clone.read(&mut buffer) {
                Ok(0) => {
                    println!(
                        "[WARNING] Connection::receive --> Connection has disconnected from {}",
                        stream.peer_addr().unwrap()
                    );
                    break;
                }
                Ok(n) => {
                    println!(
                        "[INFO] Connection::receive --> Received : {} bytes from {:?}",
                        n,
                        stream.peer_addr().unwrap()
                    );

                    let node: Node = stream.peer_addr().unwrap().into();
                    self.handle_cell(node, Cell::deserialize(&buffer));
                }
                Err(e) => {
                    println!(
                        "[FAILED] Connection::receive --> Error reading from socket: {}",
                        e
                    );
                    break;
                }
            }
        });
    }

    pub fn handle_cell(&self, node: Node, cell: Cell) {
        match CellCommand::try_from(cell.command) {
            Ok(command) => match command {
                CellCommand::Create => self.handle_create_cell(node, cell),
                _ => println!("Other"),
            },
            Err(e) => println!(
                "[FAILED] Connection::handle_cell --> Error getting cell command: {}",
                e
            ),
        };
    }

    pub fn handle_create_cell(&self, node: Node, cell: Cell) {
        let create_payload: CreatePayload = cell.payload.into();
        let aes_key = self.keys.compute_aes_key(&create_payload.dh_key);
        // self.add_aes_key(node, aes_key);
        println!("{:?}", self.tcp_streams)
    }

    pub fn establish_connection(&mut self, destination: &Node) {
        match TcpStream::connect(destination.get_addr()) {
            Ok(stream) => {
                println!(
                    "[SUCCESS] Connection::establish_connection --> Connected to Peer: {:?}",
                    destination.get_addr()
                );
                self.add_stream(*destination, stream);
            }
            Err(e) => {
                println!(
                    "[FAILED] Connection::establish_connection --> Error Connecting to Peer: {}",
                    e
                );
            }
        }
    }

    pub fn send_cell(&self, cell: &mut Cell, destination: &Node) {
        match CellCommand::try_from(cell.command) {
            Ok(command) => match command {
                CellCommand::Create => {
                    println!(
                        "[INFO] Connection::send_cell --> Sent Create Cell to: {}",
                        destination.get_info()
                    );
                }
                _ => println!("Other"),
            },
            Err(e) => println!(
                "[FAILED] Connection::handle_cell --> Error Getting Cell Command: {}",
                e
            ),
        };

        let mut stream = self.tcp_streams.read().unwrap().get(&destination).unwrap();
        let cell_serialized = cell.serialize();
        stream.write(&cell_serialized).unwrap();
    }
}

#[cfg(test)]
mod tests {

    use std::net::Ipv4Addr;

    use super::*;
    use openssl::bn::BigNumRef;

    #[test]
    fn test_connection() {
        let node1 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 7999);
        let node2 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8000);
        let node3 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001);

        let connection1 = Arc::new(Connection::new(node1));
        let connection2 = Arc::new(Connection::new(node2));
        let connection3 = Arc::new(Connection::new(node3));

        Connection::open(Arc::clone(&connection1), node1);
        Connection::open(Arc::clone(&connection2), node2);
        Connection::open(Arc::clone(&connection3), node3);

        {
            // create cell
            let public_key: &BigNumRef;
            {
                public_key = connection2.keys.dh.public_key();
            }
            let public_key_bytes = public_key.to_vec();
            let cell = &mut Cell::new_create_cell(0, Payload::new(&public_key_bytes));

            connection2.establish_connection(&node1);
            connection2.send_cell(cell, &node1);
        }

        {
            // create cell
            let public_key: &BigNumRef;
            {
                public_key = connection3.keys.dh.public_key();
            }
            let public_key_bytes = public_key.to_vec();
            let cell = &mut Cell::new_create_cell(0, Payload::new(&public_key_bytes));

            connection3.establish_connection(&node1);
            connection3.send_cell(cell, &node1);
        }

        {
            // create cell
            let public_key: &BigNumRef;
            {
                public_key = connection1.keys.dh.public_key();
            }
            let public_key_bytes = public_key.to_vec();
            let cell = &mut Cell::new_create_cell(0, Payload::new(&public_key_bytes));

            connection1.establish_connection(&node2);
            connection1.send_cell(cell, &node2);
        }

        loop {}
    }
}
