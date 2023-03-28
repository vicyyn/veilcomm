use crate::Keys;
use network::*;

use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Sender},
    thread::{self, JoinHandle},
};

pub fn listen_for_connections(node: Node, sender: Sender<ConnectionEvent>) {
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
                    .send(ConnectionEvent::NewConnection(addr.into(), stream))
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

pub fn connect_to_peer(node: Node, sender: Sender<ConnectionEvent>) -> Option<Connection> {
    match TcpStream::connect(node.get_addr()) {
        Ok(stream) => {
            println!(
                "[SUCCESS] Connection::establish_connection --> Connected to Peer: {:?}",
                node.get_addr()
            );
            sender
                .send(ConnectionEvent::NewConnection(
                    node,
                    stream.try_clone().unwrap(),
                ))
                .unwrap();
            Some(Connection::new(stream, sender.clone()))
        }
        Err(e) => {
            println!(
                "[FAILED] Connection::establish_connection --> Error Connecting to Peer: {}",
                e
            );
            None
        }
    }
}

fn start_peer(main_node: Node) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut peers: HashMap<Node, Connection> = HashMap::new();
        let (events_sender, events_receiver) = channel();
        let mut keys = Keys::new();
        let mut pending: Vec<Node> = vec![];

        listen_for_connections(main_node, events_sender.clone());

        loop {
            let connection_event = events_receiver.recv().unwrap();
            match connection_event {
                ConnectionEvent::NewConnection(node, stream) => {
                    let connection =
                        Connection::new(stream.try_clone().unwrap(), events_sender.clone());

                    peers.insert(node, connection);
                }
                ConnectionEvent::ReceiveCell(node, cell) => {
                    match CellCommand::try_from(cell.command) {
                        Ok(command) => match command {
                            CellCommand::Ping => {
                                println!("Received Ping!");
                                let connection = peers.get(&node).unwrap();
                                connection.write(Cell::new_pong_cell());
                            }
                            CellCommand::Pong => {
                                println!("Received Pong!");
                            }
                            CellCommand::Create => {
                                println!("Received Create!");
                                let create_payload: CreatePayload = cell.payload.into();
                                let aes_key = keys.compute_aes_key(&create_payload.dh_key);
                                keys.add_aes_key(node, aes_key);
                                println!(
                                    "shared secret (AES) : {:?}",
                                    hex::encode(aes_key.get_key())
                                );

                                let public_key_bytes = keys.dh.public_key().to_vec();
                                let cell =
                                    Cell::new_created_cell(0, Payload::new(&public_key_bytes));
                                let connection = peers.get(&node).unwrap();
                                connection.write(cell);
                            }
                            CellCommand::Created => {
                                println!("Received Created!");
                                let created_payload: CreatedPayload = cell.payload.into();
                                let aes_key = keys.compute_aes_key(&created_payload.dh_key);
                                keys.add_aes_key(node, aes_key);
                                println!(
                                    "shared secret (AES) : {:?}",
                                    hex::encode(aes_key.get_key())
                                );

                                if !pending.is_empty() {
                                    let extended_payload: ExtendedPayload = created_payload.into();
                                    let extended_cell =
                                        Cell::new_extended_cell(0, extended_payload);
                                    let node = pending.pop().unwrap();
                                    let connection = peers.get(&node).unwrap();
                                    connection.write(extended_cell);
                                }
                            }
                            CellCommand::Extend => {
                                println!("Received Extend Cell!");
                                let extend_payload: ExtendPayload = cell.payload.into();
                                let next_node = extend_payload.get_node();
                                let connection = connect_to_peer(next_node, events_sender.clone());

                                let create_payload: CreatePayload = extend_payload.into();
                                let cell = Cell::new_create_cell(
                                    0,
                                    Payload::new(&create_payload.serialize()),
                                );

                                connection.unwrap().write(cell);
                                pending.push(node);
                            }
                            CellCommand::Extended => {
                                println!("Received Extended Cell!");
                                let extended_payload: ExtendedPayload = cell.payload.into();
                                let aes_key = keys.compute_aes_key(&extended_payload.dh_key);
                                keys.add_aes_key(node, aes_key);
                                println!(
                                    "shared secret (AES) : {:?}",
                                    hex::encode(aes_key.get_key())
                                );
                            }
                            _ => println!("Other"),
                        },
                        Err(e) => println!(
                            "[FAILED] Connection::handle_cell --> Error getting cell command: {}",
                            e
                        ),
                    }
                }
                _ => {}
            }
        }
    })
}

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     start_peer(Node::new(
//         Ipv4Addr::new(127, 0, 0, 1),
//         args[1].parse().unwrap(),
//     ))
//     .join()
//     .unwrap();
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_tor() {
        let t1 = start_peer(Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001));
        let t3 = start_peer(Node::new(Ipv4Addr::new(127, 0, 0, 1), 8002));
        let t2 = start_peer(Node::new(Ipv4Addr::new(127, 0, 0, 1), 8000));
        let t4 = start_peer(Node::new(Ipv4Addr::new(127, 0, 0, 1), 8003));
        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
        t4.join().unwrap();
    }
}
