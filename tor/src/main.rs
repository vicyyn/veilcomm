use network::*;
use tor::{Keys, TorEvent};

use core::time;
use std::{
    collections::HashMap,
    env,
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

pub fn listen_for_connections(node: Node, sender: Sender<ConnectionEvent>) {
    thread::spawn(move || loop {
        let socket = TcpListener::bind(node.get_addr())
            .expect("[FAILED] tor::listen_for_connections --> Error while binding TcpSocket to specified addr");

        match socket.accept() {
            Ok((stream, addr)) => {
                println!(
                    "[SUCCESS] tor::listen_for_connections - New client connected: {:?}",
                    addr
                );
                sender
                    .send(ConnectionEvent::NewConnection(addr.into(), stream))
                    .unwrap()
            }
            Err(e) => {
                println!(
                    "[FAILED] tor::listen_for_connections - Error accepting client connection: {}",
                    e
                );
            }
        }
    });
}

pub fn connect_to_peer(node: Node, sender: Sender<ConnectionEvent>) {
    match TcpStream::connect(node.get_addr()) {
        Ok(stream) => {
            println!(
                "[SUCCESS] tor::connect_to_peer --> Connected to Peer: {:?}",
                node.get_addr()
            );
            sender
                .send(ConnectionEvent::NewConnection(
                    node,
                    stream.try_clone().unwrap(),
                ))
                .unwrap();
        }
        Err(e) => {
            println!(
                "[FAILED] tor::connect_to_peer --> Error Connecting to Peer: {}",
                e
            );
        }
    }
}

pub fn start_client(
    tor_events_receiver: Receiver<TorEvent>,
    connection_events_sender: Sender<ConnectionEvent>,
    peers: Arc<RwLock<HashMap<Node, Connection>>>,
    keys: Arc<RwLock<Keys>>,
) {
    println!("[INFO] tor::start_client --> Started new tor client");
    std::thread::spawn(move || loop {
        let tor_event = tor_events_receiver.recv().unwrap();
        match tor_event {
            TorEvent::Connect(node) => {
                connect_to_peer(node, connection_events_sender.clone());
            }
            TorEvent::SendExtend(node, next_node) => {
                let peers_lock = peers.read().unwrap();
                let connection = peers_lock.get(&node).unwrap();

                let public_key_bytes = keys.read().unwrap().dh.public_key().to_vec();
                let cell =
                    Cell::new_extend_cell(0, ExtendPayload::new(next_node, &public_key_bytes));

                connection.write(cell);
            }
        }
    });
}

fn process_connection_event(
    connection_event: ConnectionEvent,
    peers: Arc<RwLock<HashMap<Node, Connection>>>,
    connection_events_sender: Sender<ConnectionEvent>,
    keys: Arc<RwLock<Keys>>,
    pending: Arc<RwLock<Vec<Node>>>,
) {
    std::thread::spawn(move || match connection_event {
        ConnectionEvent::NewConnection(node, stream) => {
            println!("[INFO] tor::process_connection_event --> New connection event");
            let connection = Connection::new(
                stream.try_clone().unwrap(),
                connection_events_sender.clone(),
            );

            let mut peers_lock = peers.write().unwrap();
            peers_lock.insert(node, connection);
        }
        ConnectionEvent::ReceiveCell(node, cell) => {
            print!("[INFO] tor::process_connection_event --> New receive cell event - ");
            match CellCommand::try_from(cell.command) {
                Ok(command) => match command {
                    CellCommand::Ping => {
                        println!("Received Ping!");
                        let peers_lock = peers.read().unwrap();
                        let connection = peers_lock.get(&node).unwrap();
                        connection.write(Cell::new_pong_cell());
                    }
                    CellCommand::Pong => {
                        println!("Received Pong!");
                    }
                    CellCommand::Create => {
                        println!("Received Create!");
                        let create_payload: CreatePayload = cell.payload.into();
                        let aes_key = keys.read().unwrap().compute_aes_key(&create_payload.dh_key);
                        keys.write().unwrap().add_aes_key(node, aes_key);
                        let public_key_bytes = keys.read().unwrap().dh.public_key().to_vec();
                        let cell = Cell::new_created_cell(0, Payload::new(&public_key_bytes));
                        let peers_lock = peers.read().unwrap();
                        let connection = peers_lock.get(&node).unwrap();
                        connection.write(cell);
                    }
                    CellCommand::Created => {
                        println!("Received Created!");
                        let created_payload: CreatedPayload = cell.payload.into();
                        let aes_key = keys
                            .read()
                            .unwrap()
                            .compute_aes_key(&created_payload.dh_key);
                        keys.write().unwrap().add_aes_key(node, aes_key);
                        if !pending.read().unwrap().is_empty() {
                            let extended_payload: ExtendedPayload = created_payload.into();
                            let extended_cell = Cell::new_extended_cell(0, extended_payload);
                            let node = pending.write().unwrap().pop().unwrap();
                            let peers_lock = peers.read().unwrap();
                            let connection = peers_lock.get(&node).unwrap();
                            connection.write(extended_cell);
                        }
                    }
                    CellCommand::Extend => {
                        println!("Received Extend Cell!");
                        let extend_payload: ExtendPayload = cell.payload.into();
                        let next_node = extend_payload.get_node();
                        connect_to_peer(next_node, connection_events_sender.clone());

                        let mut connection;
                        loop {
                            let peers_lock = peers.read().unwrap();
                            connection = peers_lock.get(&next_node);
                            if connection.is_some() {
                                let create_payload: CreatePayload = extend_payload.into();
                                let cell = Cell::new_create_cell(
                                    0,
                                    Payload::new(&create_payload.serialize()),
                                );
                                connection.unwrap().write(cell);
                                pending.write().unwrap().push(node);
                                break;
                            }
                            println!("[WARNING] tor::process_connection_event --> (Extend) Error getting connection (retrying in 1000ms...)");
                            thread::sleep(time::Duration::from_millis(1000));
                        }
                    }
                    CellCommand::Extended => {
                        println!("Received Extended Cell!");
                        let extended_payload: ExtendedPayload = cell.payload.into();

                        let aes_key = keys
                            .read()
                            .unwrap()
                            .compute_aes_key(&extended_payload.dh_key);

                        keys.write().unwrap().add_aes_key(node, aes_key);
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
    });
}

fn start_peer(main_node: Node) -> Sender<TorEvent> {
    let (tor_events_sender, tor_events_receiver) = channel();
    let peers: Arc<RwLock<HashMap<Node, Connection>>> = Arc::new(RwLock::new(HashMap::new()));
    let keys = Arc::new(RwLock::new(Keys::new()));
    let (connection_events_sender, connection_events_receiver) = channel();
    let pending: Arc<RwLock<Vec<Node>>> = Arc::new(RwLock::new(vec![]));

    listen_for_connections(main_node, connection_events_sender.clone());
    start_client(
        tor_events_receiver,
        connection_events_sender.clone(),
        Arc::clone(&peers),
        Arc::clone(&keys),
    );

    std::thread::spawn(move || loop {
        let connection_event = connection_events_receiver.recv().unwrap();
        process_connection_event(
            connection_event,
            Arc::clone(&peers),
            connection_events_sender.clone(),
            Arc::clone(&keys),
            Arc::clone(&pending),
        );
    });
    return tor_events_sender.clone();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let _ = start_peer(Node::new(
        Ipv4Addr::new(127, 0, 0, 1),
        args[1].parse().unwrap(),
    ));
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tor() {
        let node1 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001);
        let node2 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8002);
        let node3 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8003);
        let node4 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8004);

        let t1 = start_peer(node1);
        let t2 = start_peer(node2);
        let t3 = start_peer(node3);
        let t4 = start_peer(node4);

        t1.send(TorEvent::Connect(node2)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        t1.send(TorEvent::SendExtend(node2, node3)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        loop {}
    }
}
