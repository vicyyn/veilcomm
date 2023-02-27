use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::ssl::{
    SslAcceptor, SslConnector, SslFiletype, SslMethod, SslStream, SslVerifyMode, SslVersion,
};

use openssl::dh::Dh;
use openssl::error::ErrorStack;

use openssl::x509::{X509Builder, X509Name, X509};
use std::collections::HashMap;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::str::FromStr;

use crate::*;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Connection {
    pub socket: TcpListener,
    pub node: Node,
    pub dh: Dh<Private>,
    pub streams: HashMap<Node, Arc<Mutex<TcpStream>>>,
}

impl Connection {
    pub fn new(node: Node) -> Self {
        let socket = TcpListener::bind(node.get_addr())
            .expect("[FAILED] Connection::new --> Error while binding TcpSocket to specified addr");

        println!(
            "[SUCCESS] Connection::open --> Listening for incoming connections: {:?}",
            node.get_addr()
        );

        Self {
            socket,
            node,
            dh: Dh::get_2048_256().unwrap().generate_key().unwrap(),
            streams: HashMap::new(),
        }
    }

    pub fn add_stream(&mut self, node: Node, stream: Arc<Mutex<TcpStream>>) {
        self.streams.insert(node, stream);
    }

    pub fn open(connection: Arc<Mutex<Self>>) {
        thread::spawn(move || {
            let mut connection_mutex = connection.lock().unwrap();
            loop {
                match connection_mutex.socket.accept() {
                    Ok((stream, addr)) => {
                        println!(
                            "[SUCCESS] Connection::open --> New client connected: {:?}",
                            addr
                        );

                        let stream_mutex = Arc::new(Mutex::new(stream));
                        connection_mutex.add_stream(
                            Node::new(
                                Ipv4Addr::from_str(&addr.ip().to_string()).unwrap(),
                                addr.port(),
                            ),
                            Arc::clone(&stream_mutex),
                        );
                        connection_mutex.receive(stream_mutex);
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

    pub fn receive(&self, stream: Arc<Mutex<TcpStream>>) {
        let mut buffer = [0u8; CELL_SIZE];
        let mut stream_mutex = stream.lock().unwrap();
        loop {
            match stream_mutex.read(&mut buffer) {
                Ok(0) => {
                    break;
                }
                Ok(n) => {
                    println!(
                        "[INFO] Connection:receive --> Received : {} bytes from {:?}",
                        n,
                        stream_mutex.peer_addr().unwrap()
                    );

                    let cell = Cell::deserialize(&buffer);
                    let received_public_key_bytes = cell.payload.dh_key;

                    let received_public_key = self
                        .dh
                        .compute_key(&BigNum::from_slice(&received_public_key_bytes).unwrap())
                        .unwrap();

                    println!(
                        "[INFO] Connection::receive --> Shared secret: {}",
                        hex::encode(received_public_key)
                    );
                }
                Err(e) => {
                    println!(
                        "[FAILED] Connection::receive --> Error reading from socket: {}",
                        e
                    );
                    break;
                }
            }
        }
    }

    pub fn establish_connection(&mut self, destination: &Node) {
        match TcpStream::connect(destination.get_addr()) {
            Ok(stream) => {
                self.add_stream(*destination, Arc::new(Mutex::new(stream)));
                println!(
                    "[SUCCESS] Connection::establish_connection --> Connected to Peer: {:?}",
                    destination.get_addr()
                );
            }
            Err(e) => {
                println!("Error connecting to server: {}", e);
            }
        }
    }

    pub fn send_cell(&mut self, cell: &mut Cell, destination: &Node) {
        let stream = self.streams.get(destination).unwrap();
        let cell_serialized = cell.serialize();
        stream.lock().unwrap().write(&cell_serialized).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use openssl::bn::BigNumRef;

    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_connection() {
        let node1 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 7999);
        let node2 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8000);
        let node3 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001);

        let connection1 = Arc::new(Mutex::new(Connection::new(node1)));
        let connection2 = Arc::new(Mutex::new(Connection::new(node2)));
        let connection3 = Arc::new(Mutex::new(Connection::new(node3)));

        Connection::open(Arc::clone(&connection1));
        Connection::open(Arc::clone(&connection2));
        Connection::open(Arc::clone(&connection3));

        let mut connection2_mutex = connection2.lock().unwrap();

        // create cell
        let public_key: &BigNumRef;
        {
            public_key = connection2_mutex.dh.public_key();
        }
        let public_key_bytes = public_key.to_vec();
        let create_payload = CreatePayload::get_create_cell(&public_key_bytes);
        let cell = &mut Cell::get_create_cell(0, create_payload);

        connection2_mutex.establish_connection(&node1);
        connection2_mutex.send_cell(cell, &node1);
        connection2_mutex.send_cell(cell, &node1);
        connection2_mutex.send_cell(cell, &node1);

        //connection1.send_cell(&mut Cell::default(), &node2);
        //connection2.send_cell(&mut Cell::default(), &node3);
        //connection3.send_cell(&mut Cell::default(), &node1);
        //connection3.send_cell(&mut Cell::default(), &node2);
        //connection1.send_cell(&mut Cell::default(), &node3);
        loop {}
    }
}
