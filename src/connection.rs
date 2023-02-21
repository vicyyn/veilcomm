// UDP connection with another node in the network.
use crate::*;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;

pub struct Connection {
    pub socket: Arc<UdpSocket>,
    pub node: Node,
}

impl Connection {
    pub fn new(node: Node) -> Self {
        let socket = UdpSocket::bind(node.get_addr())
            .expect("[FAILED] Rpc::new --> Error while binding UdpSocket to specified addr");

        Self {
            socket: Arc::new(socket),
            node,
        }
    }

    pub fn open(self: Arc<Self>) {
        thread::spawn(move || {
            let mut buffer = [0u8; CELL_SIZE];
            loop {
                let (len, src_addr) = self
                    .socket
                    .recv_from(&mut buffer)
                    .expect("[FAILED] Rpc::open --> Failed to receive data from peer");

                println!("Received :  {} bytes from {}", len, src_addr);
                let cell = Cell::deserialize(&buffer);
                println!("{:?}", cell);
            }
        });
    }

    pub fn send_cell(&self, cell: &Cell, destination: Node) {
        self.socket
            .send_to(&cell.serialize(), destination.get_addr())
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
        let node3 = Node::new("127.0.0.1".to_string(), 8001);

        let connection1 = Arc::new(Connection::new(node1.clone()));
        let connection2 = Arc::new(Connection::new(node2.clone()));
        let connection3 = Arc::new(Connection::new(node3.clone()));

        Arc::clone(&connection1).open();
        Arc::clone(&connection2).open();
        Arc::clone(&connection3).open();

        connection2.send_cell(&Cell::default(), node1.clone());
        connection2.send_cell(&Cell::default(), node3.clone());
        connection3.send_cell(&Cell::default(), node1.clone());
        connection3.send_cell(&Cell::default(), node2.clone());
        connection1.send_cell(&Cell::default(), node2.clone());
        connection1.send_cell(&Cell::default(), node3.clone());
    }
}
