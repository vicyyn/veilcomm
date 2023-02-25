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
use std::net::{TcpListener, TcpStream};

// UDP connection with another node in the network.
use crate::*;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Connection {
    pub socket: TcpListener,
    pub node: Node,
    pub dh: Dh<Private>,
    pub streams: HashMap<Key, Arc<Mutex<TcpStream>>>,
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

    pub fn add_stream(&mut self, key: Key, stream: Arc<Mutex<TcpStream>>) {
        self.streams.insert(key, stream);
    }

    pub fn open(connection: Arc<Mutex<Self>>) {
        // let keys = Keys::new();

        thread::spawn(move || {
            //let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

            //let rsa = keys.relay_id_rsa.clone();
            //let pkey = PKey::from_rsa(rsa).unwrap();
            //let mut name = X509Name::builder().unwrap();
            //name.append_entry_by_nid(Nid::COMMONNAME, "foobar.com")
            //    .unwrap();
            //let name = name.build();

            //let mut builder = X509::builder().unwrap();
            //builder.set_version(2).unwrap();
            //builder.set_subject_name(&name).unwrap();
            //builder.set_issuer_name(&name).unwrap();
            //builder.set_pubkey(&pkey).unwrap();
            //builder.sign(&pkey, MessageDigest::sha256()).unwrap();

            //let certificate: X509 = builder.build();

            //acceptor.set_private_key(&pkey).unwrap();
            //acceptor.set_certificate(&certificate).unwrap();
            //acceptor.check_private_key().unwrap();
            //let acceptor = acceptor.build();

            //acceptor
            //    .set_min_proto_version(Some(SslVersion::SSL3))
            //    .unwrap();
            //let sslconf = acceptor.build().configure().unwrap();

            let mut connection_mutex = connection.lock().unwrap();
            loop {
                match connection_mutex.socket.accept() {
                    Ok((stream, addr)) => {
                        println!(
                            "[SUCCESS] Connection::open --> New client connected: {:?}",
                            addr
                        );

                        let key = Key::new((&stream).peer_addr().unwrap().to_string());
                        let stream_mutex = Arc::new(Mutex::new(stream));

                        connection_mutex.add_stream(key, Arc::clone(&stream_mutex));
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

                    let received_public_key_bytes =
                        &cell.payload.data[0..cell.payload.length.try_into().unwrap()];

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
                let key = Key::new((&stream).peer_addr().unwrap().to_string());
                self.add_stream(key, Arc::new(Mutex::new(stream)));
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

    pub fn send_cell(&mut self, cell: &mut Cell, key: &Key) {
        //let connector = SslConnector::builder(SslMethod::tls()).unwrap();
        //let sslconf = connector.build().configure().unwrap();
        let public_key = self.dh.public_key();

        let public_key_bytes = public_key.to_vec();
        cell.payload.set_data(&public_key_bytes);
        //let mut socket = sslconf.connect(&destination.get_addr(), &stream).unwrap();
        let stream = self.streams.get(key).unwrap();
        stream.lock().unwrap().write(&cell.serialize()).unwrap();

        //stream
        //    .write(&cell.serialize())
        //    .expect("[FAILED] Rpc::send_msg --> Error while sending message to specified address");
    }
}

#[cfg(test)]
mod tests {
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

        connection2_mutex.establish_connection(&node1);
        connection2_mutex.send_cell(&mut Cell::default(), &node1.key);
        connection2_mutex.send_cell(&mut Cell::default(), &node1.key);
        connection2_mutex.send_cell(&mut Cell::default(), &node1.key);

        //connection1.send_cell(&mut Cell::default(), &node2);
        //connection2.send_cell(&mut Cell::default(), &node3);
        //connection3.send_cell(&mut Cell::default(), &node1);
        //connection3.send_cell(&mut Cell::default(), &node2);
        //connection1.send_cell(&mut Cell::default(), &node3);
        loop {}
    }
}
