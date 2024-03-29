pub mod aes_key;
pub mod cell;
pub mod cell_command;
pub mod circ_ids;
pub mod circuit;
pub mod circuit_node;
pub mod circuits;
pub mod connections;
pub mod constants;
pub mod control_payload;
pub mod cookies;
pub mod directory;
pub mod introduction_points;
pub mod keys;

pub mod network;
pub mod onion_skin;
pub mod payload;
pub mod payloads;
pub mod pending_response;
pub mod pending_responses;
pub mod relay;
pub mod relay_command;
pub mod relay_payload;
pub mod relays;
pub mod rendezvous;
pub mod streams;
pub mod user_descriptor;
pub mod user_descriptors;
pub mod users;

pub use aes_key::*;
pub use cell::*;
pub use cell_command::*;
pub use circ_ids::*;
pub use circuit::*;
pub use circuit_node::*;
pub use circuits::*;
pub use connections::*;
pub use constants::*;
pub use control_payload::*;
pub use cookies::*;
pub use directory::*;
pub use introduction_points::*;
pub use keys::*;
pub use network::*;
pub use onion_skin::*;
pub use payload::*;
pub use payloads::*;
pub use pending_response::*;
pub use pending_responses::*;
pub use relay::*;
pub use relay_command::*;
pub use relay_payload::*;
pub use relays::*;
pub use rendezvous::*;
pub use streams::*;
pub use user_descriptor::*;
pub use user_descriptors::*;
pub use users::*;

use openssl::rsa::Rsa;
use openssl::symm::{decrypt, encrypt, Cipher};
use std::sync::{
    mpsc::{channel, Sender},
    Arc, RwLock,
};

use core::time;
use std::{
    net::{Ipv4Addr, TcpListener, TcpStream},
    thread::{self},
};

pub fn start_peer(main_node: Node) -> Sender<ConnectionEvent> {
    let circuits = Circuits::new();
    let streams = Streams::new();
    let connections = Connections::new();
    let pending_responses = PendingResponses::new();
    let keys = Arc::new(RwLock::new(Keys::new()));
    let relays = Relays::new();
    let user_descriptors = UserDescriptors::new();
    let cookies = Cookies::new();
    let introduction_points = IntroductionPoints::new();
    let (connection_events_sender, connection_events_receiver) = channel();
    let circ_ids = CircIds::new();
    let users = Users::new();

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

    let user_descriptor = Arc::new(RwLock::new(keys.read().unwrap().get_user_descriptor()));

    let directory_stream = connect_to_directory(new_socket_addr(8090)).unwrap();
    publish_relay(directory_stream.try_clone().unwrap(), relay);

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
                cookies.clone(),
                introduction_points.clone(),
                circ_ids.clone(),
                users.clone(),
            );
        }
    });

    connection_events_sender.clone()
}

fn listen_for_connections(node: Node, sender: Sender<ConnectionEvent>) {
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

fn connect_to_peer(node: Node, sender: Sender<ConnectionEvent>) {
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

#[allow(clippy::too_many_arguments)]
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
    cookies: Cookies,
    introduction_points: IntroductionPoints,
    _circ_ids: CircIds,
    users: Users,
) {
    std::thread::spawn(move || match connection_event {
        ConnectionEvent::Introduce1(circ_id) => {
            println!("[INFO] tor::process_connection_event --> Introduce1 event");

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let rendezvous_point = op_circuit.get_successors().last().unwrap().node;

                let user_descriptor = user_descriptors.get_user_descriptor([0; 32]).unwrap();
                let rsa_public = Rsa::public_key_from_der(&user_descriptor.publickey).unwrap();
                let half_dh_bytes = keys.read().unwrap().dh.public_key().to_vec();
                let aes = generate_random_aes_key();
                let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());

                let introduce1 = Introduce1Payload::new(
                    generate_random_address(),
                    rendezvous_point,
                    [0; 20],
                    onion_skin,
                );
                let relay_payload = RelayPayload::new_introduce1_payload(introduce1);
                let cell = Cell::new_relay_cell(circ_id, relay_payload);
                let encrypted_cell = op_circuit.encrypt_cell(cell);
                let connection = connections.get(op_circuit.get_first().node).unwrap();

                connection.write(encrypted_cell);
                pending_responses.insert(circ_id, PendingResponse::IntroduceAck(rendezvous_point));
            }
        }
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
        ConnectionEvent::EstablishIntro(circ_id) => {
            println!("[INFO] tor::process_connection_event --> Establish intro event");

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let connection = connections.get(op_circuit.get_first().node).unwrap();
                let establish_intro = EstablishIntroPayload::new(generate_random_address());
                let relay_payload = RelayPayload::new_establish_intro_payload(establish_intro);
                let cell = Cell::new_relay_cell(circ_id, relay_payload);

                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(
                    circ_id,
                    PendingResponse::IntroEstablished(
                        op_circuit.get_successors().last().unwrap().node,
                    ),
                );
            }
        }
        ConnectionEvent::EstablishRendPoint(circ_id) => {
            println!("[INFO] tor::process_connection_event --> Establish rend point event");

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let connection = connections.get(op_circuit.get_first().node).unwrap();
                let establish_rend_point = EstablishRendPointPayload::new([0; 20]);
                let relay_payload =
                    RelayPayload::new_establish_rend_point_payload(establish_rend_point);
                let cell = Cell::new_relay_cell(circ_id, relay_payload);

                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(
                    circ_id,
                    PendingResponse::RendPointEstablished(
                        op_circuit.get_successors().last().unwrap().node,
                    ),
                );
            }
        }
        ConnectionEvent::OpenStream(circ_id, stream_node, stream_id) => {
            println!("[INFO] tor::process_connection_event --> Open stream event");

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let connection = connections.get(op_circuit.get_first().node).unwrap();
                let begin_payload = BeginPayload::new(stream_node);
                let relay_payload = RelayPayload::new_begin_payload(stream_id, begin_payload);
                let cell = Cell::new_relay_cell(circ_id, relay_payload);

                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(circ_id, PendingResponse::Connected(0));
            }
        }
        ConnectionEvent::SendCell(cell) => {
            println!("[INFO] tor::process_connection_event --> Send cell event");

            if let Circuit::OpCircuit(op_circuit) = circuits.get(cell.circ_id).unwrap() {
                let new_cell: Cell;
                if let Some(user) = users.get([0; 32]) {
                    let aes_key = user.0;
                    let mut relay_payload: RelayPayload = cell.payload.into();

                    let user_encrypted_data = encrypt(
                        Cipher::aes_128_ctr(),
                        &aes_key.get_key(),
                        None,
                        &relay_payload.data,
                    )
                    .unwrap();

                    relay_payload.data = user_encrypted_data.try_into().unwrap();

                    new_cell = Cell::new_relay_cell(cell.circ_id, relay_payload);
                } else {
                    new_cell = cell;
                }

                let connection = connections.get(op_circuit.get_first().node).unwrap();

                let encrypted_cell = op_circuit.encrypt_cell(new_cell);
                connection.write(encrypted_cell);
            }
        }
        ConnectionEvent::SendExtend(circ_id, next_node) => {
            println!("[INFO] tor::process_connection_event --> Send extend event");

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let connection = connections.get(op_circuit.get_first().node).unwrap();

                // get destination rsa publickey
                let destination_relay = relays.get_relay(next_node.get_addr()).unwrap();
                let rsa_public = Rsa::public_key_from_der(&destination_relay.identity_key).unwrap();

                // create the cell
                let half_dh_bytes = keys.read().unwrap().dh.public_key().to_vec();
                let aes = generate_random_aes_key();
                let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
                let extend_payload = ExtendPayload::new(next_node, onion_skin);
                let relay_payload = RelayPayload::new_extend_payload(extend_payload);
                let cell = Cell::new_relay_cell(circ_id, relay_payload);

                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(circ_id, PendingResponse::Extended(next_node));
            }
        }
        ConnectionEvent::SendCreate(circ_id, node) => {
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
            let cell = Cell::new_create_cell(circ_id, control_payload);

            pending_responses.insert(circ_id, PendingResponse::Created(None));
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
                        let cell = Cell::new_created_cell(cell.circ_id, control_payload);

                        let connection = connections.get(node).unwrap();
                        connection.write(cell);
                    }
                    CellCommand::Created => {
                        println!("Received Created");

                        let created_payload: CreatedPayload = cell.payload.into_created().unwrap();

                        let pending_response = pending_responses.get(cell.circ_id);
                        if pending_response.is_some() {
                            if let PendingResponse::Created(Some(return_node)) =
                                pending_response.unwrap()
                            {
                                let extended_payload: ExtendedPayload = created_payload.into();
                                let relay_payload: RelayPayload =
                                    RelayPayload::new_extended_payload(extended_payload);
                                let extended_cell = Cell::new_relay_cell(
                                    cell.circ_id,
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
                            pending_responses.pop(cell.circ_id);
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

                        if let Ok(command) = RelayCommand::try_from(relay_payload.command) {
                            match command {
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
                                            pending_responses.insert(
                                                cell.circ_id,
                                                PendingResponse::Created(Some(node)),
                                            );
                                            let create_payload: CreatePayload =
                                                extend_payload.into();
                                            let control_payload: ControlPayload =
                                                ControlPayload::new_create_payload(create_payload);
                                            let cell = Cell::new_create_cell(
                                                cell.circ_id,
                                                control_payload,
                                            );
                                            connection.unwrap().write(cell);
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

                                    if let PendingResponse::Extended(extended_node) =
                                        pending_responses.pop(cell.circ_id).unwrap()
                                    {
                                        // add successor to op circuit
                                        circuits.add_successor(
                                            cell.circ_id,
                                            CircuitNode::new(Some(aes_key), extended_node),
                                        );
                                    }
                                }
                                RelayCommand::EstablishIntro => {
                                    println!("Received Establish Intro Cell");
                                    let establish_intro_payload: EstablishIntroPayload =
                                        relay_payload.into_establish_intro();

                                    println!(
                                        "[SUCCESS] INTRODUCTION ADDRESS --> {:?}",
                                        hex::encode(establish_intro_payload.address)
                                    );

                                    introduction_points
                                        .insert(establish_intro_payload.address, cell.circ_id);

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
                                    let pending_response =
                                        pending_responses.pop(cell.circ_id).unwrap();
                                    if let PendingResponse::IntroEstablished(intro_node) =
                                        pending_response
                                    {
                                        user_descriptor
                                            .write()
                                            .unwrap()
                                            .introduction_points
                                            .push(intro_node);
                                        pending_responses.pop(cell.circ_id);
                                    }
                                }
                                RelayCommand::EstablishRendPoint => {
                                    println!("Received Establish Rend Point Cell");
                                    let establish_rend_point_payload: EstablishRendPointPayload =
                                        relay_payload.into_establish_rend_point();

                                    println!(
                                        "[SUCCESS] REND POINT COOKIE --> {:?}",
                                        hex::encode(establish_rend_point_payload.cookie)
                                    );

                                    cookies.insert(
                                        Cookie(establish_rend_point_payload.cookie),
                                        cell.circ_id,
                                    );

                                    let circuit = circuits.get(cell.circ_id).unwrap();
                                    let connection = connections.get(node).unwrap();

                                    let encrypted_relay_payload: RelayPayload = circuit
                                        .get_predecessor()
                                        .unwrap()
                                        .encrypt_payload(
                                            RelayPayload::new_rend_point_established_payload()
                                                .into(),
                                        )
                                        .into();
                                    let cell =
                                        Cell::new_relay_cell(cell.circ_id, encrypted_relay_payload);
                                    connection.write(cell);
                                }

                                RelayCommand::RendPointEstablished => {
                                    println!("Received Rend Point Established Cell");
                                    let pending_response =
                                        pending_responses.pop(cell.circ_id).unwrap();
                                    if let PendingResponse::RendPointEstablished(node) =
                                        pending_response
                                    {
                                        println!("[SUCCESS] Rend Point Established --> {:?}", node);
                                        pending_responses.pop(cell.circ_id);
                                    }
                                }
                                RelayCommand::Begin => {
                                    println!("Received Begin Cell");
                                    let begin_payload: BeginPayload = relay_payload.into_begin();
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
                                        relay_payload.into_connected();
                                    let stream_node = connected_payload.get_node();
                                    if let Some(PendingResponse::Connected(stream_id)) =
                                        pending_responses.pop(cell.circ_id)
                                    {
                                        if stream_id == relay_payload.stream_id {
                                            streams.insert(relay_payload.stream_id, stream_node);
                                        }
                                    };
                                    pending_responses.pop(cell.circ_id);
                                }
                                RelayCommand::Introduce1 => {
                                    println!("Received Introduce1 Cell");
                                    let introduce1_payload = relay_payload.into_introduce1();

                                    if let Some(stream_node) = streams.get(relay_payload.stream_id)
                                    {
                                        let circ_id = cell.circ_id;
                                        // send introduce ack back to client
                                        let circuit = circuits.get(circ_id).unwrap();
                                        let introduce_ack = RelayPayload::new_introduce_ack_payload(
                                            IntroduceAckPayload::new(0),
                                        );
                                        let new_cell = Cell::new_relay_cell(circ_id, introduce_ack);
                                        let new_cell = circuit.handle_cell(node, new_cell);
                                        let connection = connections.get(node).unwrap();
                                        connection.write(new_cell);

                                        let connection = connections.get(stream_node).unwrap();
                                        let cell = Cell::new_relay_cell(circ_id, relay_payload);
                                        connection.write(cell);
                                    } else {
                                        let circ_id = introduction_points
                                            .get(introduce1_payload.address)
                                            .unwrap();
                                        let circuit = circuits.get(circ_id).unwrap();
                                        let relay_payload = RelayPayload::new_introduce2_payload(
                                            introduce1_payload.into(),
                                        );
                                        let encrypted_payload = circuit
                                            .get_predecessor()
                                            .unwrap()
                                            .encrypt_payload(relay_payload.into());

                                        let cell =
                                            Cell::new_relay_cell(circ_id, encrypted_payload.into());

                                        let node = circuit.get_predecessor().unwrap().node;
                                        let connection = connections.get(node).unwrap();
                                        connection.write(cell);
                                    }
                                }
                                RelayCommand::IntroduceAck => {
                                    print!("Received IntroduceAck Cell");
                                    let introduce_ack_payload = relay_payload.into_introduce_ack();
                                    println!(
                                        "[SUCCESS] Introduce Complete, Status : {}",
                                        introduce_ack_payload.status
                                    );
                                    pending_responses.pop(cell.circ_id);
                                }
                                RelayCommand::Introduce2 => {
                                    println!("Received Introduce2 Cell");
                                    let introduce2_payload = relay_payload.into_introduce2();
                                    // create circuit
                                    let node9 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8009);
                                    let node10 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8010);
                                    let node11 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8011);

                                    let aes_key = keys.read().unwrap().compute_aes_key(
                                        &introduce2_payload
                                            .onion_skin
                                            .get_dh(keys.read().unwrap().user_private.clone()),
                                    );

                                    println!(
                                        "[SUCCESS] Handshake Complete With User --> AES key {:?}",
                                        hex::encode(aes_key.get_key())
                                    );

                                    // let circ_id = circuits.get_unused_circ_id();
                                    create_circuit(
                                        cell.circ_id,
                                        connection_events_sender.clone(),
                                        node9,
                                        node10,
                                        node11,
                                    );
                                    // establish stream to rendezvous point
                                    let rendezvous_point = introduce2_payload.get_node();
                                    println!(" - - - - - - -");
                                    connection_events_sender
                                        .send(ConnectionEvent::OpenStream(
                                            cell.circ_id,
                                            rendezvous_point,
                                            1,
                                        ))
                                        .unwrap();
                                    thread::sleep(time::Duration::from_millis(4000));

                                    // send rendezvous1
                                    println!(" - - - - - - -");
                                    let half_dh_bytes =
                                        keys.read().unwrap().dh.public_key().to_vec();
                                    let rendezvous1 = Rendezvous1Payload::new(
                                        introduce2_payload.cookie,
                                        half_dh_bytes.try_into().unwrap(),
                                    );
                                    let relay_payload =
                                        RelayPayload::new_rendezvous1_payload(rendezvous1, 1);
                                    let cell =
                                        Cell::new_relay_cell(cell.circ_id, relay_payload.clone());

                                    // send rendezvous1
                                    connection_events_sender
                                        .send(ConnectionEvent::SendCell(cell.clone()))
                                        .unwrap();
                                    thread::sleep(time::Duration::from_millis(4000));

                                    users.insert(
                                        [0; 32],
                                        aes_key,
                                        cell.circ_id,
                                        relay_payload.stream_id,
                                    );
                                }
                                RelayCommand::Rendezvous1 => {
                                    println!("Received Rendezvous1 Cell");
                                    let rendezvous1_payload = relay_payload.into_rendezvous1();

                                    if let Some(stream_node) = streams.get(relay_payload.stream_id)
                                    {
                                        let circ_id = cell.circ_id;
                                        let connection = connections.get(stream_node).unwrap();
                                        let cell = Cell::new_relay_cell(circ_id, relay_payload);
                                        connection.write(cell);
                                    } else {
                                        if let Some(circ_id) =
                                            cookies.get(rendezvous1_payload.cookie.into())
                                        {
                                            streams.insert(circ_id, node);

                                            let circuit = circuits.get(circ_id).unwrap();
                                            let relay_payload =
                                                RelayPayload::new_rendezvous2_payload(
                                                    rendezvous1_payload.into(),
                                                );
                                            let encrypted_payload = circuit
                                                .get_predecessor()
                                                .unwrap()
                                                .encrypt_payload(relay_payload.into());
                                            let cell = Cell::new_relay_cell(
                                                circ_id,
                                                encrypted_payload.into(),
                                            );
                                            let node = circuit.get_predecessor().unwrap().node;
                                            let connection = connections.get(node).unwrap();
                                            connection.write(cell);
                                        }
                                        streams.insert(3, node);
                                    }
                                }
                                RelayCommand::Rendezvous2 => {
                                    println!("Received Rendezvous2 Cell");
                                    let rendezvous2_payload = relay_payload.into_rendezvous2();
                                    let aes_key = keys
                                        .read()
                                        .unwrap()
                                        .compute_aes_key(&rendezvous2_payload.dh_key);
                                    println!(
                                        "[SUCCESS] Handshake Complete With User --> AES key {:?}",
                                        hex::encode(aes_key.get_key())
                                    );
                                    users.insert(
                                        [0; 32],
                                        aes_key,
                                        cell.circ_id,
                                        relay_payload.stream_id,
                                    )
                                }
                                RelayCommand::Data => {
                                    println!("Received Data Cell");

                                    if let Some(user) = users.get([0; 32]) {
                                        let user_decrypted_data = decrypt(
                                            Cipher::aes_128_ctr(),
                                            &user.0.get_key(),
                                            None,
                                            &relay_payload.data,
                                        )
                                        .unwrap();

                                        if let Ok(message) = String::from_utf8(user_decrypted_data)
                                        {
                                            println!(
                                "[INFO] tor::process_connection_event --> Received Message : {message}",
                            );
                                        }
                                    }

                                    if let Some(stream_node) = streams.get(relay_payload.stream_id)
                                    {
                                        let connection = connections.get(stream_node).unwrap();
                                        let cell =
                                            Cell::new_relay_cell(cell.circ_id, relay_payload);
                                        connection.write(cell);
                                    } else if let Some(Circuit::OrCircuit(or_circuit)) =
                                        circuits.get(cell.circ_id)
                                    {
                                        let encrypted_payload = or_circuit
                                            .get_predecessor()
                                            .encrypt_payload(relay_payload.into());
                                        let cell = Cell::new_relay_cell(
                                            cell.circ_id,
                                            encrypted_payload.into(),
                                        );
                                        let node = or_circuit.get_predecessor().node;
                                        let connection = connections.get(node).unwrap();
                                        connection.write(cell);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => println!("Other"),
                },
                Err(e) => {
                    println!("[FAILED] Connection::handle_cell --> Error getting cell command: {e}",)
                }
            }
        }
    });
}

pub fn create_circuit(
    circ_id: u16,
    t: Sender<ConnectionEvent>,
    first_hop: Node,
    second_hop: Node,
    third_hop: Node,
) {
    println!(" - -- - - - -");
    t.send(ConnectionEvent::FetchFromDirectory).unwrap();
    thread::sleep(time::Duration::from_millis(4000));

    println!(" - -- - - - -");
    t.send(ConnectionEvent::Connect(first_hop)).unwrap();
    thread::sleep(time::Duration::from_millis(1000));

    println!(" - -- - - - -");
    t.send(ConnectionEvent::SendCreate(circ_id, first_hop))
        .unwrap();
    thread::sleep(time::Duration::from_millis(1000));

    println!(" - -- - - - -");
    t.send(ConnectionEvent::SendExtend(circ_id, second_hop))
        .unwrap();
    thread::sleep(time::Duration::from_millis(4000));

    println!(" - -- - - - -");
    t.send(ConnectionEvent::SendExtend(circ_id, third_hop))
        .unwrap();
    thread::sleep(time::Duration::from_millis(4000));
}
