use network::*;
use tor::{Circuit, CircuitNode, Keys, PendingResponse};

use core::time;
use std::{
    collections::HashMap,
    env,
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Sender},
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

fn process_connection_event(
    connection_event: ConnectionEvent,
    peers: Arc<RwLock<HashMap<Node, Connection>>>,
    connection_events_sender: Sender<ConnectionEvent>,
    keys: Arc<RwLock<Keys>>,
    pending: Arc<RwLock<HashMap<Node, PendingResponse>>>,
    circuits: Arc<RwLock<HashMap<u16, Circuit>>>,
    constructed_circuit: Arc<RwLock<Vec<CircuitNode>>>,
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
        ConnectionEvent::Connect(node) => {
            println!("[INFO] tor::process_connection_event --> Connect event");
            connect_to_peer(node, connection_events_sender.clone());
        }
        ConnectionEvent::SendCell(node, cell) => {
            let peers_lock = peers.read().unwrap();
            let connection = peers_lock.get(&node).unwrap();

            let constructed_circuit_lock = constructed_circuit.read().unwrap();
            let mut encrypted_cell = cell.clone();
            for circuit_node in constructed_circuit_lock.iter().rev() {
                encrypted_cell.payload = circuit_node.encrypt_payload(cell.payload.clone());
            }

            connection.write(encrypted_cell);
        }
        ConnectionEvent::SendExtend(node, next_node) => {
            let peers_lock = peers.read().unwrap();
            let connection = peers_lock.get(&node).unwrap();
            let public_key_bytes = keys.read().unwrap().dh.public_key().to_vec();

            let extend_payload = ExtendPayload::new(next_node, &public_key_bytes);
            let relay_payload = RelayPayload::new_extend_payload(extend_payload);
            let cell = Cell::new_extend_cell(0, relay_payload);

            let mut pending_lock = pending.write().unwrap();
            pending_lock.insert(node, PendingResponse::Extended);
            connection.write(cell);
        }
        ConnectionEvent::SendCreate(node) => {
            let peers_lock = peers.read().unwrap();
            let connection = peers_lock.get(&node).unwrap();
            let public_key_bytes = keys.read().unwrap().dh.public_key().to_vec();

            let create_payload = CreatePayload::new(&public_key_bytes);
            let control_payload = ControlPayload::new_create_payload(create_payload);
            let cell = Cell::new_create_cell(0, control_payload);

            pending
                .write()
                .unwrap()
                .insert(node, PendingResponse::Created(None));
            connection.write(cell);
        }
        ConnectionEvent::ReceiveCell(node, cell) => {
            print!("[INFO] tor::process_connection_event --> New receive cell event - ");
            match CellCommand::try_from(cell.command) {
                Ok(command) => match command {
                    // CellCommand::Ping => {
                    //     println!("Received Ping!");
                    //     let peers_lock = peers.read().unwrap();
                    //     let connection = peers_lock.get(&node).unwrap();
                    //     pending.write().unwrap().insert(node, PendingResponse::Pong);
                    //     connection.write(Cell::new_pong_cell());
                    // }
                    // CellCommand::Pong => {
                    //     println!("Received Pong!");
                    // }
                    CellCommand::Create => {
                        println!("Received Create");
                        let create_payload: CreatePayload = cell.payload.into_create().unwrap();
                        let aes_key = keys.read().unwrap().compute_aes_key(&create_payload.dh_key);
                        let public_key_bytes = keys.read().unwrap().dh.public_key().to_vec();

                        let mut circuit_lock = circuits.write().unwrap();
                        let predecessor_circuit_node =
                            CircuitNode::new(cell.circ_id, Some(aes_key), node);
                        let circuit = Circuit::new(predecessor_circuit_node, None);
                        circuit_lock.insert(cell.circ_id, circuit);

                        let created_payload = CreatedPayload::new(&public_key_bytes);
                        let control_payload = ControlPayload::new_created_payload(created_payload);
                        let cell = Cell::new_created_cell(0, control_payload);

                        let peers_lock = peers.read().unwrap();
                        let connection = peers_lock.get(&node).unwrap();
                        connection.write(cell);
                    }
                    CellCommand::Created => {
                        println!("Received Created");
                        let created_payload: CreatedPayload = cell.payload.into_created().unwrap();
                        let aes_key = keys
                            .read()
                            .unwrap()
                            .compute_aes_key(&created_payload.dh_key);
                        let mut pending_lock = pending.write().unwrap();
                        if pending_lock.get(&node).is_some() {
                            if let PendingResponse::Created(Some(return_node)) =
                                pending_lock.get(&node).unwrap()
                            {
                                let extended_payload: ExtendedPayload = created_payload.into();
                                let relay_payload: RelayPayload =
                                    RelayPayload::new_extended_payload(extended_payload);
                                let extended_cell = Cell::new_extended_cell(0, relay_payload);

                                let peers_lock = peers.read().unwrap();
                                let connection = peers_lock.get(&return_node).unwrap();
                                connection.write(extended_cell);
                                pending_lock.remove(&node).unwrap();
                            }
                        }

                        let mut constructed_circuit_lock = constructed_circuit.write().unwrap();
                        constructed_circuit_lock.push(CircuitNode::new(
                            cell.circ_id,
                            Some(aes_key),
                            node,
                        ));
                    }
                    CellCommand::Relay => {
                        print!("Received Relay Cell -- ");

                        let circuits_lock = circuits.read().unwrap();
                        let circuit = circuits_lock.get(&cell.circ_id).unwrap();

                        let decrypted_payload =
                            circuit.predecessor.decrypt_payload(cell.payload.clone());
                        let relay_payload: RelayPayload = decrypted_payload.into();

                        if !relay_payload.recognized.eq(&0) {
                            println!("Forwarding Relay Cell");
                            if let Some(successor) = circuit.successor.clone() {
                                let peers_lock = peers.read().unwrap();
                                let connection = peers_lock.get(&successor.node).unwrap();
                                connection.write(cell);
                            }
                            return;
                        }

                        match RelayCommand::try_from(relay_payload.command) {
                            Ok(command) => match command {
                                RelayCommand::Extend => {
                                    println!("Received Extend Cell");
                                    let extend_payload: ExtendPayload =
                                        cell.payload.into_extend().unwrap();
                                    let next_node = extend_payload.get_node();
                                    connect_to_peer(next_node, connection_events_sender.clone());

                                    let mut circuit_lock = circuits.write().unwrap();
                                    let successor_circuit_node =
                                        CircuitNode::new(cell.circ_id, None, next_node);
                                    let circuit = circuit_lock.get_mut(&cell.circ_id).unwrap();
                                    circuit.set_successor(Some(successor_circuit_node));

                                    let mut connection;
                                    loop {
                                        let peers_lock = peers.read().unwrap();
                                        connection = peers_lock.get(&next_node);
                                        if connection.is_some() {
                                            let create_payload: CreatePayload =
                                                extend_payload.into();
                                            let control_payload: ControlPayload =
                                                ControlPayload::new_create_payload(create_payload);
                                            let cell = Cell::new_create_cell(0, control_payload);
                                            connection.unwrap().write(cell);
                                            pending.write().unwrap().insert(
                                                next_node,
                                                PendingResponse::Created(Some(node)),
                                            );
                                            break;
                                        }
                                        println!("[WARNING] tor::process_connection_event --> (Extend) Error getting connection (retrying in 1000ms...)");
                                        thread::sleep(time::Duration::from_millis(1000));
                                    }
                                }
                                RelayCommand::Extended => {
                                    println!("Received Extended Cell");
                                    let extended_payload: ExtendedPayload =
                                        cell.payload.into_extended().unwrap();

                                    let aes_key = keys
                                        .read()
                                        .unwrap()
                                        .compute_aes_key(&extended_payload.dh_key);

                                    let mut constructed_circuit_lock =
                                        constructed_circuit.write().unwrap();
                                    constructed_circuit_lock.push(CircuitNode::new(
                                        cell.circ_id,
                                        Some(aes_key),
                                        node,
                                    ));
                                }
                                RelayCommand::Data => {
                                    println!("Received Data Cell");
                                    if let Ok(message) =
                                        String::from_utf8(relay_payload.data.to_vec())
                                    {
                                        println!(
                                "[INFO] tor::process_connection_event --> Received Message : {message}",
                            );
                                    }
                                }
                                _ => {}
                            },
                            Err(e) => {}
                        }
                    }
                    _ => println!("Other"),
                },
                Err(e) => {
                    println!("[FAILED] Connection::handle_cell --> Error getting cell command: {e}",)
                }
            }
        }
        _ => {}
    });
}

fn start_peer(main_node: Node) -> Sender<ConnectionEvent> {
    let constructed_circuit: Arc<RwLock<Vec<CircuitNode>>> = Arc::new(RwLock::new(vec![]));
    let circuits: Arc<RwLock<HashMap<u16, Circuit>>> = Arc::new(RwLock::new(HashMap::new()));
    let peers: Arc<RwLock<HashMap<Node, Connection>>> = Arc::new(RwLock::new(HashMap::new()));
    let keys = Arc::new(RwLock::new(Keys::new()));
    let (connection_events_sender, connection_events_receiver) = channel();
    let pending: Arc<RwLock<HashMap<Node, PendingResponse>>> =
        Arc::new(RwLock::new(HashMap::new()));

    listen_for_connections(main_node, connection_events_sender.clone());

    std::thread::spawn({
        let connection_events_sender = connection_events_sender.clone();
        move || loop {
            let connection_event = connection_events_receiver.recv().unwrap();
            process_connection_event(
                connection_event,
                Arc::clone(&peers),
                connection_events_sender.clone(),
                Arc::clone(&keys),
                Arc::clone(&pending),
                Arc::clone(&circuits),
                Arc::clone(&constructed_circuit),
            );
        }
    });

    return connection_events_sender.clone();
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

        t1.send(ConnectionEvent::Connect(node2)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        t1.send(ConnectionEvent::SendCreate(node2)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        t1.send(ConnectionEvent::SendExtend(node2, node3)).unwrap();
        thread::sleep(time::Duration::from_millis(5000));

        let relay_payload = RelayPayload::new_data_payload("Hello!".as_bytes());
        let cell = Cell::new_extend_cell(0, relay_payload);
        t1.send(ConnectionEvent::SendCell(node2, cell)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        loop {}
    }
}
