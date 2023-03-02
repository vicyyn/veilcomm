use openssl::bn::BigNum;

use crate::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Connection {
    pub node: Node,
    pub keys: Keys,
    pub tcp_streams: HashMap<Node, Arc<Mutex<TcpStream>>>,
}

impl Connection {
    pub fn new(node: Node) -> Self {
        Self {
            node,
            keys: Keys::new(),
            tcp_streams: HashMap::new(),
        }
    }

    pub fn add_stream(&mut self, node: Node, stream: Arc<Mutex<TcpStream>>) {
        self.tcp_streams.insert(node, stream);
    }

    pub fn add_aes_key(&mut self, node: Node, aes_key: AESKey) {
        self.keys.aes_keys.insert(node, aes_key);
    }

    pub fn open(connection: Arc<Mutex<Self>>, node: Node) {
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
                    Ok((stream, addr)) => {
                        println!(
                            "[SUCCESS] Connection::open --> New client connected: {:?}",
                            addr
                        );

                        let node = stream.local_addr().unwrap();
                        let mut connection_mutex = connection.lock().unwrap();
                        let stream_mutex = Arc::new(Mutex::new(stream));
                        connection_mutex.add_stream(node.into(), Arc::clone(&stream_mutex));
                        Connection::receive(Arc::clone(&connection), stream_mutex);
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

    pub fn receive(connection: Arc<Mutex<Self>>, stream: Arc<Mutex<TcpStream>>) {
        let mut buffer = [0u8; CELL_SIZE];

        thread::spawn(move || loop {
            let mut stream_mutex = stream.lock().unwrap();
            match stream_mutex.read(&mut buffer) {
                Ok(0) => {
                    println!(
                        "[WARNING] Connection::receive --> Connection has disconnected from {}",
                        stream_mutex.peer_addr().unwrap()
                    );
                    break;
                }
                Ok(n) => {
                    println!(
                        "[INFO] Connection::receive --> Received : {} bytes from {:?}",
                        n,
                        stream_mutex.peer_addr().unwrap()
                    );

                    let node: Node = stream_mutex.peer_addr().unwrap().into();
                    let mut connection_mutex = connection.lock().unwrap();
                    connection_mutex.handle_cell(node, Cell::deserialize(&buffer));
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

    pub fn handle_cell(&mut self, node: Node, cell: Cell) {
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

    pub fn handle_create_cell(&mut self, node: Node, cell: Cell) {
        let create_payload: CreatePayload = cell.payload.into();
        let aes_key = self.keys.compute_aes_key(&create_payload.dh_key);
        self.add_aes_key(node, aes_key);
        println!("{:?}", self.tcp_streams)
    }

    pub fn establish_connection(&mut self, destination: &Node) {
        match TcpStream::connect(destination.get_addr()) {
            Ok(stream) => {
                println!(
                    "[SUCCESS] Connection::establish_connection --> Connected to Peer: {:?}",
                    destination.get_addr()
                );
                self.add_stream(*destination, Arc::new(Mutex::new(stream)));
            }
            Err(e) => {
                println!(
                    "[FAILED] Connection::establish_connection --> Error Connecting to Peer: {}",
                    e
                );
            }
        }
    }

    pub fn send_cell(&mut self, cell: &mut Cell, destination: &Node) {
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

        let stream = self.tcp_streams.get(destination).unwrap();
        let cell_serialized = cell.serialize();
        stream.lock().unwrap().write(&cell_serialized).unwrap();
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

        let connection1 = Arc::new(Mutex::new(Connection::new(node1)));
        let connection2 = Arc::new(Mutex::new(Connection::new(node2)));
        let connection3 = Arc::new(Mutex::new(Connection::new(node3)));

        Connection::open(Arc::clone(&connection1), node1);
        Connection::open(Arc::clone(&connection2), node2);
        Connection::open(Arc::clone(&connection3), node3);

        {
            let mut connection2_mutex = connection2.lock().unwrap();

            // create cell
            let public_key: &BigNumRef;
            {
                public_key = connection2_mutex.keys.dh.public_key();
            }
            let public_key_bytes = public_key.to_vec();
            let cell = &mut Cell::new_create_cell(0, Payload::new(&public_key_bytes));

            connection2_mutex.establish_connection(&node1);
            connection2_mutex.send_cell(cell, &node1);
        }

        {
            let mut connection3_mutex = connection3.lock().unwrap();

            // create cell
            let public_key: &BigNumRef;
            {
                public_key = connection3_mutex.keys.dh.public_key();
            }
            let public_key_bytes = public_key.to_vec();
            let cell = &mut Cell::new_create_cell(0, Payload::new(&public_key_bytes));

            connection3_mutex.establish_connection(&node1);
            connection3_mutex.send_cell(cell, &node1);
        }

        {
            let mut connection1_mutex = connection1.lock().unwrap();

            // create cell
            let public_key: &BigNumRef;
            {
                public_key = connection1_mutex.keys.dh.public_key();
            }
            let public_key_bytes = public_key.to_vec();
            let cell = &mut Cell::new_create_cell(0, Payload::new(&public_key_bytes));

            connection1_mutex.establish_connection(&node2);
            connection1_mutex.send_cell(cell, &node2);
        }

        loop {}
    }
}
