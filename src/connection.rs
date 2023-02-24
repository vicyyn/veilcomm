use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::PKey;
use openssl::ssl::{
    SslAcceptor, SslConnector, SslFiletype, SslMethod, SslStream, SslVerifyMode, SslVersion,
};
use openssl::x509::{X509Builder, X509Name, X509};
use std::net::{TcpListener, TcpStream};

// UDP connection with another node in the network.
use crate::*;
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;

pub struct Connection {
    pub socket: Arc<TcpListener>,
    pub node: Node,
}

impl Connection {
    pub fn new(node: Node) -> Self {
        let socket = TcpListener::bind(node.get_addr())
            .expect("[FAILED] Rpc::new --> Error while binding UdpSocket to specified addr");

        Self {
            socket: Arc::new(socket),
            node,
        }
    }

    pub fn open(self: Arc<Self>) {
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
            for stream in self.socket.incoming() {
                match stream {
                    Ok(mut stream) => {
                        println!("a receiver is connected");
                        //thread::spawn(move || {
                        //let mut stream = acceptor.accept(stream).unwrap();
                        self.receive(&mut stream);
                        //});
                    }
                    Err(_e) => {
                        println! {"connection failed"}
                    }
                }
            }
        });
    }

    pub fn receive(&self, socket: &mut TcpStream) {
        let mut buffer = [0u8; CELL_SIZE];
        loop {
            let len = socket
                .read(&mut buffer)
                .expect("[FAILED] Rpc::open --> Failed to receive data from peer");

            println!("Received : {} bytes from {:?}", len, socket);
            let cell = Cell::deserialize(&buffer);
            println!("{:?}", cell);
        }
    }

    pub fn send_cell(&self, cell: &Cell, destination: &Node) {
        //let connector = SslConnector::builder(SslMethod::tls()).unwrap();
        //let sslconf = connector.build().configure().unwrap();
        let mut stream = TcpStream::connect(destination.get_addr()).unwrap();

        //let mut socket = sslconf.connect(&destination.get_addr(), &stream).unwrap();

        stream
            .write(&cell.serialize())
            .expect("[FAILED] Rpc::send_msg --> Error while sending message to specified address");
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

        let connection1 = Arc::new(Connection::new(node1));
        let connection2 = Arc::new(Connection::new(node2));
        let connection3 = Arc::new(Connection::new(node3));

        Arc::clone(&connection1).open();
        Arc::clone(&connection2).open();
        Arc::clone(&connection3).open();

        connection2.send_cell(&Cell::default(), &node1);
        connection2.send_cell(&Cell::default(), &node3);
        connection3.send_cell(&Cell::default(), &node1);
        connection3.send_cell(&Cell::default(), &node2);
        connection1.send_cell(&Cell::default(), &node2);
        connection1.send_cell(&Cell::default(), &node3);
    }
}
