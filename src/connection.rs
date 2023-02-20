// UDP connection with another node in the network.
use crate::*;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::str;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct Connection {
    pub socket: Arc<UdpSocket>,
    pub pending: Arc<Mutex<HashMap<Key, mpsc::Sender<Option<Cell>>>>>,
    pub node: Node,
}

impl Connection {
    pub fn new(node: Node) -> Self {
        let socket = UdpSocket::bind(node.get_addr())
            .expect("[FAILED] Rpc::new --> Error while binding UdpSocket to specified addr");

        Self {
            socket: Arc::new(socket),
            pending: Arc::new(Mutex::new(HashMap::new())),
            node,
        }
    }

    pub fn open(connection: Connection) {
        thread::spawn(move || {
            let mut buf = [0u8; 512];

            loop {
                let (len, src_addr) = connection
                    .socket
                    .recv_from(&mut buf)
                    .expect("[FAILED] Rpc::open --> Failed to receive data from peer");

                let mut decoded: Cell = bincode::deserialize(&buf.to_vec())
                    .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload");

                println!("{:?}", decoded);
            }
        });
    }

    pub fn send_cell(&self, cell: &Cell, destination: Node) {
        let encoded = bincode::serialize(cell)
            .expect("[FAILED] Rpc::send_msg --> Unable to serialize message");

        self.socket
            .send_to(&encoded, destination.get_addr())
            .expect("[FAILED] Rpc::send_msg --> Error while sending message to specified address");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection() {
        let node1 = Node::new("127.0.0.1".to_string(), 7999);
        let node2 = Node::new("127.0.0.1".to_string(), 8000);

        let connection1 = Connection::new(node1.clone());
        let connection2 = Connection::new(node2.clone());

        Connection::open(connection1);

        connection2.send_cell(&Cell::default(), node1);
        loop {}
    }
}
