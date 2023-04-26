use directory::{
    connect_to_directory, fetch_relays, fetch_user_descriptors, new_socket_addr,
    publish_user_descriptor, Relay, Relays, UserDescriptor, UserDescriptors,
};
use network::*;
use openssl::rsa::Rsa;
use tor::*;

use core::time;
use std::{
    env,
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock,
    },
    thread::{self},
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
    connections: Connections,
    pending_responses: PendingResponses,
    connection_events_sender: Sender<ConnectionEvent>,
    keys: Arc<RwLock<Keys>>,
    circuits: Circuits,
    relays: Relays,
    user_descriptors: UserDescriptors,
    streams: Streams,
    directory_stream: TcpStream,
    user_descriptor: Arc<RwLock<UserDescriptor>>,
) {
    std::thread::spawn(move || match connection_event {
        ConnectionEvent::NewConnection(node, stream) => {
            println!("[INFO] tor::process_connection_event --> New connection event");
            let connection = Connection::new(
                stream.try_clone().unwrap(),
                connection_events_sender.clone(),
            );

            connections.insert(node, connection);
        }
        ConnectionEvent::Connect(node) => {
            println!("[INFO] tor::process_connection_event --> Connect event");
            connect_to_peer(node, connection_events_sender.clone());
        }
        ConnectionEvent::EstablishIntro(node) => {
            println!("[INFO] tor::process_connection_event --> Establish intro event");
            let connection = connections.get(node).unwrap();
            // TODO GENERATE USER ADDRESS,
            // TODO MAKE AN ABSTRACT METHOD THAT CREATES AN INTRODUCTION POINT USING 3 HOPS (YOU
            // ONLY PROVIDE THE NODE YOU WANT AND IT CREATES THE CIRCUIT BEFOREHAND)
            let establish_intro = EstablishIntroPayload::new(generate_random_address());
            let relay_payload = RelayPayload::new_establish_intro_payload(establish_intro);
            let cell = Cell::new_relay_cell(0, relay_payload);

            if let Circuit::OpCircuit(op_circuit) = circuits.get(cell.circ_id).unwrap() {
                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(
                    node,
                    PendingResponse::IntroEstablished(
                        op_circuit.get_successors().last().unwrap().node,
                    ),
                );
            }
        }
        ConnectionEvent::OpenStream(node, stream_node) => {
            println!("[INFO] tor::process_connection_event --> Open stream event");
            let connection = connections.get(node).unwrap();
            let begin_payload = BeginPayload::new(stream_node);
            let relay_payload = RelayPayload::new_begin_payload(4, begin_payload);
            let cell = Cell::new_relay_cell(0, relay_payload);

            if let Circuit::OpCircuit(op_circuit) = circuits.get(cell.circ_id).unwrap() {
                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(node, PendingResponse::Connected(4));
            }
        }
        ConnectionEvent::SendCell(node, cell) => {
            println!("[INFO] tor::process_connection_event --> Send cell event");
            let connection = connections.get(node).unwrap();

            if let Circuit::OpCircuit(op_circuit) = circuits.get(cell.circ_id).unwrap() {
                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
            }
        }
        ConnectionEvent::SendExtend(node, next_node) => {
            println!("[INFO] tor::process_connection_event --> Send extend event");
            let connection = connections.get(node).unwrap();

            // get destination rsa publickey
            let destination_relay = relays.get_relay(next_node.get_addr()).unwrap();
            let rsa_public = Rsa::public_key_from_der(&destination_relay.identity_key).unwrap();

            // create the cell
            let half_dh_bytes = keys.read().unwrap().dh.public_key().to_vec();
            let aes = generate_random_aes_key();
            let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
            let extend_payload = ExtendPayload::new(next_node, onion_skin);
            let relay_payload = RelayPayload::new_extend_payload(extend_payload);
            let cell = Cell::new_relay_cell(0, relay_payload);

            if let Circuit::OpCircuit(op_circuit) = circuits.get(cell.circ_id).unwrap() {
                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(node, PendingResponse::Extended);
            }
        }
        ConnectionEvent::SendCreate(node) => {
            println!("[INFO] tor::process_connection_event --> Send create event");
            let connection = connections.get(node).unwrap();

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

            pending_responses.insert(node, PendingResponse::Created(None));
            connection.write(cell);
        }
        ConnectionEvent::PublishUserDescriptor => {
            println!("[INFO] tor::process_connection_event --> Publish user descriptor event");
            publish_user_descriptor(directory_stream, user_descriptor.read().unwrap().clone());
        }
        ConnectionEvent::FetchFromDirectory => {
            println!("[INFO] tor::process_connection_event --> Fetch from directory event");
            let _relays = fetch_relays(directory_stream.try_clone().unwrap()).unwrap();
            let _user_descriptors = fetch_user_descriptors(directory_stream).unwrap();

            relays.set(_relays);
            user_descriptors.set(_user_descriptors);
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

                        let predecessor_circuit_node = CircuitNode::new(Some(aes_key), node);
                        let circuit = Circuit::new_or_circuit(predecessor_circuit_node, None);
                        circuits.insert(cell.circ_id, circuit);

                        let created_payload = CreatedPayload::new(&public_key_bytes);
                        let control_payload = ControlPayload::new_created_payload(created_payload);
                        let cell = Cell::new_created_cell(0, control_payload);

                        let connection = connections.get(node).unwrap();
                        connection.write(cell);
                    }
                    CellCommand::Created => {
                        println!("Received Created");

                        let created_payload: CreatedPayload = cell.payload.into_created().unwrap();

                        let pending_response = pending_responses.get(node);
                        if pending_response.is_some() {
                            if let PendingResponse::Created(Some(return_node)) =
                                pending_response.unwrap()
                            {
                                let extended_payload: ExtendedPayload = created_payload.into();
                                let relay_payload: RelayPayload =
                                    RelayPayload::new_extended_payload(extended_payload);
                                let extended_cell = Cell::new_relay_cell(
                                    0,
                                    circuits
                                        .get(cell.circ_id)
                                        .unwrap()
                                        .get_predecessor()
                                        .unwrap()
                                        .encrypt_payload(relay_payload.into())
                                        .into(),
                                );

                                let connection = connections.get(return_node).unwrap();
                                connection.write(extended_cell);
                            } else {
                                let aes_key = keys
                                    .read()
                                    .unwrap()
                                    .compute_aes_key(&created_payload.dh_key);

                                println!(
                                    "[SUCCESS] Handshake Complete --> AES key {:?}",
                                    hex::encode(aes_key.get_key())
                                );

                                // create op circuit
                                let mut op_circuit = Circuit::new_op_circuit();
                                op_circuit.add_successor(CircuitNode::new(Some(aes_key), node));
                                circuits.insert(cell.circ_id, op_circuit);
                                println!("[SUCCESS] Added To Op Circuit --> Node {:?}", node);
                            }
                            pending_responses.pop(node);
                        }
                    }
                    CellCommand::Relay => {
                        print!("Received Relay Cell -- ");
                        let circuit = circuits.get(cell.circ_id).unwrap();
                        let mut relay_payload: RelayPayload = cell.payload.clone().into();

                        if !relay_payload.recognized.eq(&0) {
                            relay_payload = circuit.handle_cell(node, cell.clone()).payload.into();
                            if !relay_payload.recognized.eq(&0) {
                                println!("Forwarding Relay Cell");
                                let mut new_cell = cell.clone();
                                new_cell.payload = relay_payload.into();
                                let destination = circuit.get_cell_destination(node).unwrap();
                                let connection = connections.get(destination.node).unwrap();
                                connection.write(new_cell);
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

                                    // extend existing or circuit
                                    circuits.set_successor(
                                        cell.circ_id,
                                        Some(CircuitNode::new(None, next_node)),
                                    );

                                    let mut connection;
                                    loop {
                                        connection = connections.get(next_node);
                                        if connection.is_some() {
                                            let create_payload: CreatePayload =
                                                extend_payload.into();
                                            let control_payload: ControlPayload =
                                                ControlPayload::new_create_payload(create_payload);
                                            let cell = Cell::new_create_cell(0, control_payload);
                                            connection.unwrap().write(cell);
                                            pending_responses.insert(
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
                                        relay_payload.into_extended();

                                    let aes_key = keys
                                        .read()
                                        .unwrap()
                                        .compute_aes_key(&extended_payload.dh_key);

                                    println!(
                                        "[SUCCESS] Handshake Complete --> AES key {:?}",
                                        hex::encode(aes_key.get_key())
                                    );

                                    // add successor to op circuit
                                    circuits.add_successor(
                                        cell.circ_id,
                                        CircuitNode::new(Some(aes_key), node),
                                    );

                                    pending_responses.pop(node);
                                }
                                RelayCommand::EstablishIntro => {
                                    println!("Received Establish Intro Cell");
                                    let establish_intro_payload: EstablishIntroPayload =
                                        relay_payload.into_establish_intro();

                                    println!(
                                        "[SUCCESS] INTRODUCTION ADDRESS --> {:?}",
                                        hex::encode(establish_intro_payload.address)
                                    );

                                    let circuit = circuits.get(cell.circ_id).unwrap();
                                    let connection = connections.get(node).unwrap();
                                    let encrypted_relay_payload: RelayPayload = circuit
                                        .get_predecessor()
                                        .unwrap()
                                        .encrypt_payload(
                                            RelayPayload::new_intro_established_payload().into(),
                                        )
                                        .into();
                                    let cell =
                                        Cell::new_relay_cell(cell.circ_id, encrypted_relay_payload);
                                    connection.write(cell);
                                }
                                RelayCommand::IntroEstablished => {
                                    println!("Received Intro Established Cell");
                                    let pending_response = pending_responses.pop(node).unwrap();
                                    if let PendingResponse::IntroEstablished(intro_node) =
                                        pending_response
                                    {
                                        user_descriptor
                                            .write()
                                            .unwrap()
                                            .introduction_points
                                            .push(intro_node);
                                    }
                                }
                                RelayCommand::Begin => {
                                    println!("Received Begin Cell");
                                    let begin_payload: BeginPayload =
                                        relay_payload.into_begin_payload();
                                    let stream_node = begin_payload.get_node();
                                    let stream_id = relay_payload.stream_id;
                                    connect_to_peer(stream_node, connection_events_sender.clone());
                                    streams.insert(stream_id, stream_node);

                                    let circuit = circuits.get(cell.circ_id).unwrap();
                                    let connection = connections.get(node).unwrap();
                                    let relay_payload: RelayPayload =
                                        RelayPayload::new_connected_payload(
                                            stream_id,
                                            begin_payload.into(),
                                        );
                                    let encrypted_relay_payload: RelayPayload = circuit
                                        .get_predecessor()
                                        .unwrap()
                                        .encrypt_payload(relay_payload.into())
                                        .into();
                                    let cell =
                                        Cell::new_relay_cell(cell.circ_id, encrypted_relay_payload);
                                    connection.write(cell);
                                }
                                RelayCommand::Connected => {
                                    println!("Received Connected Cell");
                                    let connected_payload: ConnectedPayload =
                                        relay_payload.into_connected_payload();
                                    let stream_node = connected_payload.get_node();
                                    if let Some(pending_response) = pending_responses.pop(node) {
                                        if let PendingResponse::Connected(stream_id) =
                                            pending_response
                                        {
                                            if stream_id == relay_payload.stream_id {
                                                streams
                                                    .insert(relay_payload.stream_id, stream_node);
                                            }
                                        }
                                    };
                                    pending_responses.pop(node);
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

fn start_peer(main_node: Node, is_user: bool) -> Sender<ConnectionEvent> {
    let circuits = Circuits::new();
    let streams = Streams::new();
    let connections = Connections::new();
    let pending_responses = PendingResponses::new();
    let keys = Arc::new(RwLock::new(Keys::new()));
    let relays = Relays::new();
    let user_descriptors = UserDescriptors::new();
    let (connection_events_sender, connection_events_receiver) = channel();

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

    let user_descriptor = Arc::new(RwLock::new(
        keys.read().unwrap().get_user_descriptor(vec![]),
    ));

    let directory_stream = connect_to_directory(relay, new_socket_addr(8090)).unwrap();

    listen_for_connections(main_node, connection_events_sender.clone());
    std::thread::spawn({
        let connection_events_sender = connection_events_sender.clone();
        move || loop {
            let connection_event = connection_events_receiver.recv().unwrap();
            process_connection_event(
                connection_event,
                connections.clone(),
                pending_responses.clone(),
                connection_events_sender.clone(),
                Arc::clone(&keys),
                circuits.clone(),
                relays.clone(),
                user_descriptors.clone(),
                streams.clone(),
                directory_stream.try_clone().unwrap(),
                Arc::clone(&user_descriptor),
            );
        }
    });

    return connection_events_sender.clone();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let _ = start_peer(
        Node::new(Ipv4Addr::new(127, 0, 0, 1), args[1].parse().unwrap()),
        false,
    );
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
        let node5 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8005);

        thread::spawn(|| {
            start_directory(new_socket_addr(8090));
        });

        let t5 = start_peer(node5, false);
        let t4 = start_peer(node4, false);
        let t3 = start_peer(node3, false);
        let t2 = start_peer(node2, false);
        let t1 = start_peer(node1, true);

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::FetchFromDirectory).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::Connect(node2)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::SendCreate(node2)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::SendExtend(node2, node3)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::SendExtend(node2, node4)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::OpenStream(node2, node5)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::EstablishIntro(node2)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::PublishUserDescriptor).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        let relay_payload = RelayPayload::new_data_payload("Hello!".as_bytes());
        let cell = Cell::new_relay_cell(0, relay_payload);
        t1.send(ConnectionEvent::SendCell(node2, cell)).unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        loop {}
    }
}
