use directory::{new_socket_addr, Relay, Relays};
use network::*;
use openssl::{
    rand::rand_bytes,
    rsa::{Padding, Rsa},
};
use tor::{Circuit, CircuitNode, Keys, PendingResponse};

use core::time;
use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock,
    },
    thread,
};

fn generate_random_aes_key() -> [u8; 16] {
    let mut key = [0u8; 16];
    rand_bytes(&mut key).unwrap();
    key
}

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
    relays: Arc<Relays>,
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

            let circuits_lock = circuits.read().unwrap();
            let circuit = circuits_lock.get(&cell.circ_id).unwrap();
            let encryption_nodes = circuit.get_encryption_nodes().unwrap();

            let mut encrypted_cell = cell.clone();
            for circuit_node in encryption_nodes.iter().rev() {
                encrypted_cell.payload = circuit_node.encrypt_payload(cell.payload.clone());
            }

            connection.write(encrypted_cell);
        }
        ConnectionEvent::SendExtend(node, next_node) => {
            // get connection
            let peers_lock = peers.read().unwrap();
            let connection = peers_lock.get(&node).unwrap();

            // get destination rsa publickey
            let destination_relay = relays.get_relay(next_node.get_addr()).unwrap();
            let rsa_public = Rsa::public_key_from_der(&destination_relay.identity_key).unwrap();

            // create the cell
            let half_dh_bytes = keys.read().unwrap().dh.public_key().to_vec();
            let aes = generate_random_aes_key();
            let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
            let extend_payload = ExtendPayload::new(next_node, onion_skin);
            let relay_payload = RelayPayload::new_extend_payload(extend_payload);
            let mut cell = Cell::new_extend_cell(0, relay_payload);

            let circuits_lock = circuits.read().unwrap();
            let circuit = circuits_lock.get(&cell.circ_id).unwrap();
            let encryption_nodes = circuit.get_encryption_nodes().unwrap();

            for circuit_node in encryption_nodes.iter().rev() {
                cell.payload = circuit_node.encrypt_payload(cell.payload.clone());
            }

            let mut pending_lock = pending.write().unwrap();
            pending_lock.insert(node, PendingResponse::Extended);
            connection.write(cell);
        }
        ConnectionEvent::SendCreate(node) => {
            // get connection
            let peers_lock = peers.read().unwrap();
            let connection = peers_lock.get(&node).unwrap();

            // get destination rsa publickey
            let destination_relay = relays.get_relay(node.get_addr()).unwrap();
            let rsa_public = Rsa::public_key_from_der(&destination_relay.identity_key).unwrap();

            // create the cell
            let half_dh_bytes = keys.read().unwrap().dh.public_key().to_vec();
            let aes = generate_random_aes_key();
            let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
            let create_payload = CreatePayload::new(onion_skin);
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
                    CellCommand::Create => {
                        println!("Received Create");
                        let create_payload: CreatePayload = cell.payload.into_create().unwrap();
                        let aes_key = keys.read().unwrap().compute_aes_key(
                            &create_payload
                                .onion_skin
                                .get_dh(keys.read().unwrap().relay_id_rsa.clone()),
                        );
                        println!(
                            "[SUCCESS] Handshake Complete --> AES key {:?}",
                            hex::encode(aes_key.get_key())
                        );

                        let public_key_bytes = keys.read().unwrap().dh.public_key().to_vec();

                        let mut circuit_lock = circuits.write().unwrap();
                        let predecessor_circuit_node =
                            CircuitNode::new(cell.circ_id, Some(aes_key), node);
                        let circuit = Circuit::new_or_circuit(predecessor_circuit_node, None);
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

                        println!(
                            "[SUCCESS] Handshake Complete --> AES key {:?}",
                            hex::encode(aes_key.get_key())
                        );

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

                        let mut circuits_lock = circuits.write().unwrap();
                        let mut op_circuit = Circuit::new_op_circuit();
                        op_circuit.add_successor(CircuitNode::new(
                            cell.circ_id,
                            Some(aes_key),
                            node,
                        ));
                        circuits_lock.insert(cell.circ_id, op_circuit).unwrap();
                    }
                    CellCommand::Relay => {
                        print!("Received Relay Cell -- ");
                        let mut circuits_lock = circuits.write().unwrap();
                        let circuit = circuits_lock.get(&cell.circ_id).unwrap();

                        let mut relay_payload: RelayPayload = cell.payload.clone().into();

                        if circuit.is_or_circuit() {
                            let decrypted_payload = circuit
                                .get_decryption_node()
                                .unwrap()
                                .decrypt_payload(cell.payload.clone());
                            relay_payload = decrypted_payload.clone().into();

                            if !relay_payload.recognized.eq(&0) {
                                println!("Forwarding Relay Cell");
                                let mut cell = cell.clone();
                                cell.payload = decrypted_payload;
                                if let Some(successor) = circuit.get_successor() {
                                    let peers_lock = peers.read().unwrap();
                                    let connection = peers_lock.get(&successor.node).unwrap();
                                    connection.write(cell);
                                }
                                return;
                            }
                        }

                        match RelayCommand::try_from(relay_payload.command) {
                            Ok(command) => match command {
                                RelayCommand::Extend => {
                                    println!("Received Extend Cell");
                                    let extend_payload: ExtendPayload = relay_payload.into_extend();
                                    let next_node = extend_payload.get_node();
                                    connect_to_peer(next_node, connection_events_sender.clone());

                                    let successor_circuit_node =
                                        CircuitNode::new(cell.circ_id, None, next_node);
                                    let circuit = circuits_lock.get_mut(&cell.circ_id).unwrap();
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

                                    println!(
                                        "[SUCCESS] Handshake Complete --> AES key {:?}",
                                        hex::encode(aes_key.get_key())
                                    );

                                    let mut circuits_lock = circuits.write().unwrap();
                                    let mut op_circuit = Circuit::new_op_circuit();
                                    op_circuit.add_successor(CircuitNode::new(
                                        cell.circ_id,
                                        Some(aes_key),
                                        node,
                                    ));
                                    circuits_lock.insert(cell.circ_id, op_circuit).unwrap();
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

pub fn connect_to_directory(relay: Relay, address: SocketAddr) -> Result<Relays, ()> {
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            println!(
                "[SUCCESS] tor::connect_to_directory --> Connected to Directory: {:?}",
                address
            );
            stream.write(&relay.serialize()).unwrap();
            println!("[SUCCESS] tor::connect_to_directory --> Sent server descriptor to directory");

            let mut buffer = [0u8; 10240];
            let mut stream = stream.try_clone().unwrap();
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!(
                        "[WARNING] tor::connect_to_directory --> Connection has disconnected from {}",
                        stream.peer_addr().unwrap()
                    );
                }
                Ok(n) => {
                    println!(
                        "[SUCCESS] tor::connect_to_directory --> Received : {} bytes from {:?}",
                        n,
                        stream.peer_addr().unwrap()
                    );

                    let relays = Relays::deserialize(&buffer);
                    println!(
                        "[SUCCESS] tor::connect_to_directory --> Received {} Relay",
                        relays.len()
                    );
                    return Ok(relays);
                }
                Err(e) => {
                    println!(
                        "[FAILED] tor::connect_to_directory --> Error reading from socket: {}",
                        e
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "[FAILED] tor::connect_to_peer --> Error Connecting to Peer: {}",
                e
            );
        }
    }
    Err(())
}

fn start_peer(main_node: Node) -> Sender<ConnectionEvent> {
    let circuits: Arc<RwLock<HashMap<u16, Circuit>>> = Arc::new(RwLock::new(HashMap::new()));
    let peers: Arc<RwLock<HashMap<Node, Connection>>> = Arc::new(RwLock::new(HashMap::new()));
    let keys = Arc::new(RwLock::new(Keys::new()));
    let (connection_events_sender, connection_events_receiver) = channel();
    let pending: Arc<RwLock<HashMap<Node, PendingResponse>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let relay = Relay::new(
        "Joe".to_string(),
        keys.read()
            .unwrap()
            .relay_id_rsa
            .public_key_to_der()
            .unwrap(),
        main_node.get_addr(),
        "joe@gmail.com".to_string(),
    );

    let relays = Arc::new(connect_to_directory(relay, new_socket_addr(8090)).unwrap());

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
                Arc::clone(&relays),
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
    use directory::{new_socket_addr, start_directory};

    #[test]
    fn test_tor() {
        let node1 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001);
        let node2 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8002);
        let node3 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8003);
        let node4 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8004);

        start_directory(new_socket_addr(8090));

        let t4 = start_peer(node4);
        let t3 = start_peer(node3);
        let t2 = start_peer(node2);
        let t1 = start_peer(node1);

        t1.send(ConnectionEvent::Connect(node2)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        t1.send(ConnectionEvent::SendCreate(node2)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        t1.send(ConnectionEvent::SendExtend(node2, node3)).unwrap();
        thread::sleep(time::Duration::from_millis(2000));

        let relay_payload = RelayPayload::new_data_payload("Hello!".as_bytes());
        let cell = Cell::new_extend_cell(0, relay_payload);
        t1.send(ConnectionEvent::SendCell(node2, cell)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        loop {}
    }
}
