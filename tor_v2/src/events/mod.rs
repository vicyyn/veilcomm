pub mod tor_event;
pub mod utils;
pub use utils::*;
pub mod tor_change;

use crate::*;
use directory::UserDescriptor;
use openssl::rsa::Rsa;
use openssl::symm::{decrypt, encrypt, Cipher};
pub use tor_event::*;

use core::time;
use std::{
    net::Ipv4Addr,
    sync::{mpsc::Sender, Arc, RwLock},
    thread::{self},
};

pub fn process_tor_event(
    tor_event: TorEvent,
    connections: Connections,
    pending_responses: PendingResponses,
    tor_event_sender: Sender<TorEvent>,
    tor_change_sender: Sender<TorChange>,
    keys: Arc<RwLock<Keys>>,
    circuits: Circuits,
    relays: RelayDescriptors,
    user_descriptors: UserDescriptors,
    streams: Streams,
    user_descriptor: Arc<RwLock<UserDescriptor>>,
    cookies: Cookies,
    introduction_points: IntroductionPoints,
    _circ_ids: CircIds,
    users: Users,
) {
    std::thread::spawn(move || match tor_event {
        TorEvent::Introduce1(circ_id) => {
            println!("[INFO] tor::process_connection_event --> Introduce1 event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Introduce1 event".to_string(),
                ))
                .unwrap();

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let rendezvous_point = op_circuit.get_successors().last().unwrap().socket_address;

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
                let connection = connections
                    .get(op_circuit.get_first().socket_address)
                    .unwrap();

                connection.write(encrypted_cell);
                pending_responses.insert(circ_id, PendingResponse::IntroduceAck(rendezvous_point));
            }
        }
        TorEvent::NewConnection(node, stream) => {
            println!("[INFO] tor::process_connection_event --> New connection event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> New connection event".to_string(),
                ))
                .unwrap();
            let connection = Connection::new(stream.try_clone().unwrap(), tor_event_sender.clone());

            connections.insert(node, connection);
        }
        TorEvent::Connect(node) => {
            println!("[INFO] tor::process_connection_event --> Connect event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Connect event".to_string(),
                ))
                .unwrap();
            connect_to_peer(node, tor_event_sender.clone());
        }
        TorEvent::EstablishIntro(circ_id) => {
            println!("[INFO] tor::process_connection_event --> Establish intro event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Establish intro event".to_string(),
                ))
                .unwrap();

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let connection = connections
                    .get(op_circuit.get_first().socket_address)
                    .unwrap();
                let establish_intro = EstablishIntroPayload::new(generate_random_address());
                let relay_payload = RelayPayload::new_establish_intro_payload(establish_intro);
                let cell = Cell::new_relay_cell(circ_id, relay_payload);

                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(
                    circ_id,
                    PendingResponse::IntroEstablished(
                        op_circuit.get_successors().last().unwrap().socket_address,
                    ),
                );
            }
        }
        TorEvent::EstablishRendPoint(circ_id) => {
            println!("[INFO] tor::process_connection_event --> Establish rend point event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Establish rend point event"
                        .to_string(),
                ))
                .unwrap();

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let connection = connections
                    .get(op_circuit.get_first().socket_address)
                    .unwrap();
                let establish_rend_point = EstablishRendPointPayload::new([0; 20]);
                let relay_payload =
                    RelayPayload::new_establish_rend_point_payload(establish_rend_point);
                let cell = Cell::new_relay_cell(circ_id, relay_payload);

                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(
                    circ_id,
                    PendingResponse::RendPointEstablished(
                        op_circuit.get_successors().last().unwrap().socket_address,
                    ),
                );
            }
        }
        TorEvent::OpenStream(circ_id, stream_node, stream_id) => {
            println!("[INFO] tor::process_connection_event --> Open stream event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Open stream event".to_string(),
                ))
                .unwrap();

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let connection = connections
                    .get(op_circuit.get_first().socket_address)
                    .unwrap();
                let begin_payload = BeginPayload::new(stream_node);
                let relay_payload = RelayPayload::new_begin_payload(stream_id, begin_payload);
                let cell = Cell::new_relay_cell(circ_id, relay_payload);

                let encrypted_cell = op_circuit.encrypt_cell(cell);
                connection.write(encrypted_cell);
                pending_responses.insert(circ_id, PendingResponse::Connected(0));
            }
        }
        TorEvent::SendCell(cell) => {
            println!("[INFO] tor::process_connection_event --> Send cell event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Send cell event".to_string(),
                ))
                .unwrap();

            if let Circuit::OpCircuit(op_circuit) = circuits.get(cell.circ_id).unwrap() {
                let new_cell: Cell;
                if let Some(user) = users.get([0; 32]) {
                    let aes_key = user.0;
                    let mut relay_payload: RelayPayload = cell.payload.into();

                    let user_encrypted_data =
                        encrypt(Cipher::aes_128_ctr(), &aes_key, None, &relay_payload.data)
                            .unwrap();

                    relay_payload.data = user_encrypted_data.try_into().unwrap();

                    new_cell = Cell::new_relay_cell(cell.circ_id, relay_payload);
                } else {
                    new_cell = cell;
                }

                let connection = connections
                    .get(op_circuit.get_first().socket_address)
                    .unwrap();

                let encrypted_cell = op_circuit.encrypt_cell(new_cell);
                connection.write(encrypted_cell);
            }
        }
        TorEvent::SendExtend(circ_id, next_node) => {
            println!("[INFO] tor::process_connection_event --> Send extend event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Send extend event".to_string(),
                ))
                .unwrap();

            if let Circuit::OpCircuit(op_circuit) = circuits.get(circ_id).unwrap() {
                let connection = connections
                    .get(op_circuit.get_first().socket_address)
                    .unwrap();

                // get destination rsa publickey
                let destination_relay = relays.get_relay(next_node).unwrap();
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
        TorEvent::SendCreate(circ_id, node) => {
            println!("[INFO] tor::process_connection_event --> Send create event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Send create event".to_string(),
                ))
                .unwrap();

            let connection = connections.get(node).unwrap();
            // get destination rsa publickey
            let destination_relay = relays.get_relay(node).unwrap();
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
        TorEvent::PublishUserDescriptor => {
            println!("[INFO] tor::process_connection_event --> Publish user descriptor event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Publish user descriptor event"
                        .to_string(),
                ))
                .unwrap();
            let user_descriptor = user_descriptor.read().unwrap().clone();
            publish_user(user_descriptor);
        }
        TorEvent::FetchFromDirectory => {
            println!("[INFO] tor::process_connection_event --> Fetch from directory event");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> Fetch from directory event"
                        .to_string(),
                ))
                .unwrap();
            let _relays = RelayDescriptors::new_from(get_relays());
            let _user_descriptors = UserDescriptors::new_from(get_users());

            relays.set(_relays);
            user_descriptors.set(_user_descriptors);
        }
        TorEvent::ReceiveCell(node, cell) => {
            print!("[INFO] tor::process_connection_event --> New receive cell event - ");
            tor_change_sender
                .send(TorChange::Logs(
                    "[INFO] tor::process_connection_event --> New receive cell event - "
                        .to_string(),
                ))
                .unwrap();
            match CellCommand::try_from(cell.command) {
                Ok(command) => match command {
                    CellCommand::Create => {
                        println!("Received Create");
                        tor_change_sender
                            .send(TorChange::Logs("Received Create".to_string()))
                            .unwrap();
                        let create_payload: CreatePayload = cell.payload.into_create().unwrap();
                        let aes_key = keys.read().unwrap().compute_aes_key(
                            &create_payload
                                .onion_skin
                                .get_dh(keys.read().unwrap().relay_id_rsa.clone()),
                        );
                        println!(
                            "[SUCCESS] Handshake Complete --> AES key {:?}",
                            hex::encode(aes_key)
                        );
                        tor_change_sender
                            .send(TorChange::Logs(format!(
                                "[SUCCESS] Handshake Complete --> AES key {:?}",
                                hex::encode(aes_key)
                            )))
                            .unwrap();

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
                        tor_change_sender
                            .send(TorChange::Logs("Received Created".to_string()))
                            .unwrap();

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
                                    hex::encode(aes_key)
                                );
                                tor_change_sender
                                    .send(TorChange::Logs(format!(
                                        "[SUCCESS] Handshake Complete --> AES key {:?}",
                                        hex::encode(aes_key)
                                    )))
                                    .unwrap();

                                // create op circuit
                                let mut op_circuit = Circuit::new_op_circuit();
                                op_circuit.add_successor(CircuitNode::new(Some(aes_key), node));
                                circuits.insert(cell.circ_id, op_circuit);
                                println!("[SUCCESS] Added To Op Circuit --> Node {:?}", node);
                                tor_change_sender
                                    .send(TorChange::Logs(format!(
                                        "[SUCCESS] Added To Op Circuit --> Node {:?}",
                                        node
                                    )))
                                    .unwrap();
                            }
                            pending_responses.pop(cell.circ_id);
                        }
                    }
                    CellCommand::Relay => {
                        print!("Received Relay Cell -- ");
                        tor_change_sender
                            .send(TorChange::Logs("Received Relay Cell -- ".to_string()))
                            .unwrap();

                        let circuit = circuits.get(cell.circ_id).unwrap();
                        let mut relay_payload: RelayPayload = cell.payload.clone().into();

                        if !relay_payload.recognized.eq(&0) {
                            relay_payload = circuit.handle_cell(node, cell.clone()).payload.into();
                            if !relay_payload.recognized.eq(&0) {
                                println!("Forwarding Relay Cell");
                                tor_change_sender
                                    .send(TorChange::Logs("Forwarding Relay Cell".to_string()))
                                    .unwrap();

                                let mut new_cell = cell.clone();
                                new_cell.payload = relay_payload.into();
                                let destination = circuit.get_cell_destination(node).unwrap();
                                let connection =
                                    connections.get(destination.socket_address).unwrap();
                                connection.write(new_cell);
                                return;
                            }
                        }

                        match RelayCommand::try_from(relay_payload.command) {
                            Ok(command) => match command {
                                RelayCommand::Extend => {
                                    println!("Received Extend Cell");
                                    tor_change_sender
                                        .send(TorChange::Logs("Received Extend Cell".to_string()))
                                        .unwrap();

                                    let extend_payload: ExtendPayload = relay_payload.into_extend();
                                    let next_node = extend_payload.get_address();
                                    connect_to_peer(next_node, tor_event_sender.clone());

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
                                        tor_change_sender
                                            .send(TorChange::Logs("[WARNING] tor::process_connection_event --> (Extend) Error getting connection (retrying in 1000ms...)".to_string()))
                                            .unwrap();
                                        thread::sleep(time::Duration::from_millis(1000));
                                    }
                                }
                                RelayCommand::Extended => {
                                    println!("Received Extended Cell");
                                    tor_change_sender
                                        .send(TorChange::Logs("Received Extended Cell".to_string()))
                                        .unwrap();
                                    let extended_payload: ExtendedPayload =
                                        relay_payload.into_extended();

                                    let aes_key = keys
                                        .read()
                                        .unwrap()
                                        .compute_aes_key(&extended_payload.dh_key);

                                    println!(
                                        "[SUCCESS] Handshake Complete --> AES key {:?}",
                                        hex::encode(aes_key)
                                    );
                                    tor_change_sender
                                        .send(TorChange::Logs(format!(
                                            "[SUCCESS] Handshake Complete --> AES key {:?}",
                                            hex::encode(aes_key)
                                        )))
                                        .unwrap();

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
                                    tor_change_sender
                                        .send(TorChange::Logs(
                                            "Received Establish Intro Cell".to_string(),
                                        ))
                                        .unwrap();
                                    let establish_intro_payload: EstablishIntroPayload =
                                        relay_payload.into_establish_intro();

                                    println!(
                                        "[SUCCESS] INTRODUCTION ADDRESS --> {:?}",
                                        hex::encode(establish_intro_payload.address)
                                    );
                                    tor_change_sender
                                        .send(TorChange::Logs(format!(
                                            "[SUCCESS] INTRODUCTION ADDRESS --> {:?}",
                                            hex::encode(establish_intro_payload.address)
                                        )))
                                        .unwrap();

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
                                    tor_change_sender
                                        .send(TorChange::Logs(format!(
                                            "Received Intro Established Cell",
                                        )))
                                        .unwrap();
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
                                    tor_change_sender
                                        .send(TorChange::Logs(
                                            "Received Establish Rend Point Cell".to_string(),
                                        ))
                                        .unwrap();
                                    let establish_rend_point_payload: EstablishRendPointPayload =
                                        relay_payload.into_establish_rend_point();

                                    println!(
                                        "[SUCCESS] REND POINT COOKIE --> {:?}",
                                        hex::encode(establish_rend_point_payload.cookie)
                                    );
                                    tor_change_sender
                                        .send(TorChange::Logs(format!(
                                            "[SUCCESS] REND POINT COOKIE --> {:?}",
                                            hex::encode(establish_rend_point_payload.cookie)
                                        )))
                                        .unwrap();

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
                                    tor_change_sender
                                        .send(TorChange::Logs(
                                            "Received Rend Point Established Cell".to_string(),
                                        ))
                                        .unwrap();
                                    let pending_response =
                                        pending_responses.pop(cell.circ_id).unwrap();
                                    if let PendingResponse::RendPointEstablished(node) =
                                        pending_response
                                    {
                                        println!("[SUCCESS] Rend Point Established --> {:?}", node);
                                        tor_change_sender
                                            .send(TorChange::Logs(format!(
                                                "[SUCCESS] Rend Point Established --> {:?}",
                                                node
                                            )))
                                            .unwrap();
                                        pending_responses.pop(cell.circ_id);
                                    }
                                }
                                RelayCommand::Begin => {
                                    println!("Received Begin Cell");
                                    tor_change_sender
                                        .send(TorChange::Logs("Received Begin Cell".to_string()))
                                        .unwrap();
                                    let begin_payload: BeginPayload = relay_payload.into_begin();
                                    let stream_node = begin_payload.get_address();
                                    let stream_id = relay_payload.stream_id;
                                    connect_to_peer(stream_node, tor_event_sender.clone());
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
                                    tor_change_sender
                                        .send(TorChange::Logs(
                                            "Received Connected Cell".to_string(),
                                        ))
                                        .unwrap();
                                    let connected_payload: ConnectedPayload =
                                        relay_payload.into_connected();
                                    let stream_node = connected_payload.get_address();
                                    if let Some(pending_response) =
                                        pending_responses.pop(cell.circ_id)
                                    {
                                        if let PendingResponse::Connected(stream_id) =
                                            pending_response
                                        {
                                            if stream_id == relay_payload.stream_id {
                                                streams
                                                    .insert(relay_payload.stream_id, stream_node);
                                            }
                                        }
                                    };
                                    pending_responses.pop(cell.circ_id);
                                }
                                RelayCommand::Introduce1 => {
                                    println!("Received Introduce1 Cell");
                                    tor_change_sender
                                        .send(TorChange::Logs(
                                            ("Received Introduce1 Cell".to_string()),
                                        ))
                                        .unwrap();
                                    let introduce1_payload = relay_payload.into_introduce1();

                                    if let Some(stream_node) = streams.get(relay_payload.stream_id)
                                    {
                                        let circ_id = cell.circ_id;
                                        // send introduce ack back to client
                                        let circuit = circuits.get(circ_id).unwrap();
                                        let introduce_ack = RelayPayload::new_introduce_ack_payload(
                                            IntroduceAckPayload::new(0),
                                        );
                                        let new_cell =
                                            Cell::new_relay_cell(circ_id, introduce_ack.into());
                                        let new_cell = circuit.handle_cell(node, new_cell);
                                        let connection = connections.get(node).unwrap();
                                        connection.write(new_cell);

                                        let connection = connections.get(stream_node).unwrap();
                                        let cell =
                                            Cell::new_relay_cell(circ_id, relay_payload).into();
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

                                        let node =
                                            circuit.get_predecessor().unwrap().socket_address;
                                        let connection = connections.get(node).unwrap();
                                        connection.write(cell);
                                    }
                                }
                                RelayCommand::IntroduceAck => {
                                    print!("Received IntroduceAck Cell");
                                    tor_change_sender
                                        .send(TorChange::Logs(
                                            "Received IntroduceAck Cell".to_string(),
                                        ))
                                        .unwrap();
                                    let introduce_ack_payload = relay_payload.into_introduce_ack();
                                    println!(
                                        "[SUCCESS] Introduce Complete, Status : {}",
                                        introduce_ack_payload.status
                                    );
                                    tor_change_sender
                                        .send(TorChange::Logs(format!(
                                            "[SUCCESS] Introduce Complete, Status : {}",
                                            introduce_ack_payload.status
                                        )))
                                        .unwrap();
                                    pending_responses.pop(cell.circ_id);
                                }
                                RelayCommand::Introduce2 => {
                                    println!("Received Introduce2 Cell");
                                    tor_change_sender
                                        .send(TorChange::Logs(format!("Received Introduce2 Cell")))
                                        .unwrap();
                                    let introduce2_payload = relay_payload.into_introduce2();
                                    // create circuit
                                    let node9 =
                                        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8009);
                                    let node10 =
                                        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8010);
                                    let node11 =
                                        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8011);

                                    let aes_key = keys.read().unwrap().compute_aes_key(
                                        &introduce2_payload
                                            .onion_skin
                                            .get_dh(keys.read().unwrap().user_private.clone()),
                                    );

                                    println!(
                                        "[SUCCESS] Handshake Complete With User --> AES key {:?}",
                                        hex::encode(aes_key)
                                    );
                                    tor_change_sender
                                        .send(TorChange::Logs(format!(
                                            "[SUCCESS] Handshake Complete With User --> AES key {:?}",
                                            hex::encode(aes_key)
                                        )))
                                        .unwrap();

                                    // let circ_id = circuits.get_unused_circ_id();
                                    create_circuit(
                                        cell.circ_id,
                                        tor_event_sender.clone(),
                                        node9,
                                        node10,
                                        node11,
                                    );
                                    // establish stream to rendezvous point
                                    let rendezvous_point = introduce2_payload.get_address();
                                    println!(" - - - - - - -");
                                    tor_event_sender
                                        .send(TorEvent::OpenStream(
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
                                    tor_event_sender
                                        .send(TorEvent::SendCell(cell.clone()))
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
                                    tor_change_sender
                                        .send(TorChange::Logs(format!("Received Rendezvous1 Cell")))
                                        .unwrap();
                                    let rendezvous1_payload = relay_payload.into_rendezvous1();

                                    if let Some(stream_node) = streams.get(relay_payload.stream_id)
                                    {
                                        let circ_id = cell.circ_id;
                                        let connection = connections.get(stream_node).unwrap();
                                        let cell =
                                            Cell::new_relay_cell(circ_id, relay_payload).into();
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
                                            let node =
                                                circuit.get_predecessor().unwrap().socket_address;
                                            let connection = connections.get(node).unwrap();
                                            connection.write(cell);
                                        }
                                        streams.insert(3, node);
                                    }
                                }
                                RelayCommand::Rendezvous2 => {
                                    println!("Received Rendezvous2 Cell");
                                    tor_change_sender
                                        .send(TorChange::Logs(format!("Received Rendezvous2 Cell")))
                                        .unwrap();
                                    let rendezvous2_payload = relay_payload.into_rendezvous2();
                                    let aes_key = keys
                                        .read()
                                        .unwrap()
                                        .compute_aes_key(&rendezvous2_payload.dh_key);
                                    println!(
                                        "[SUCCESS] Handshake Complete With User --> AES key {:?}",
                                        hex::encode(aes_key)
                                    );
                                    tor_change_sender
                                        .send(TorChange::Logs(format!(
                                            "[SUCCESS] Handshake Complete With User --> AES key {:?}",
                                            hex::encode(aes_key)
                                        )))
                                        .unwrap();
                                    users.insert(
                                        [0; 32],
                                        aes_key,
                                        cell.circ_id,
                                        relay_payload.stream_id,
                                    )
                                }
                                RelayCommand::Data => {
                                    println!("Received Data Cell");
                                    tor_change_sender
                                        .send(TorChange::Logs(format!("Received Data Cell")))
                                        .unwrap();

                                    if let Some(user) = users.get([0; 32]) {
                                        let user_decrypted_data = decrypt(
                                            Cipher::aes_128_ctr(),
                                            &user.0,
                                            None,
                                            &relay_payload.data,
                                        )
                                        .unwrap();

                                        if let Ok(message) = String::from_utf8(user_decrypted_data)
                                        {
                                            println!(
                                "[INFO] tor::process_connection_event --> Received Message : {message}",
                            );
                                            tor_change_sender
                                            .send(TorChange::Logs(format!(
                                                "[INFO] tor::process_connection_event --> Received Message : {message}",
                                            )))
                                            .unwrap();
                                        }
                                    }

                                    if let Some(stream_node) = streams.get(relay_payload.stream_id)
                                    {
                                        let connection = connections.get(stream_node).unwrap();
                                        let cell =
                                            Cell::new_relay_cell(cell.circ_id, relay_payload)
                                                .into();
                                        connection.write(cell);
                                    } else {
                                        if let Some(circuit) = circuits.get(cell.circ_id) {
                                            if let Circuit::OrCircuit(or_circuit) = circuit {
                                                let encrypted_payload = or_circuit
                                                    .get_predecessor()
                                                    .encrypt_payload(relay_payload.into());
                                                let cell = Cell::new_relay_cell(
                                                    cell.circ_id,
                                                    encrypted_payload.into(),
                                                );
                                                let node =
                                                    or_circuit.get_predecessor().socket_address;
                                                let connection = connections.get(node).unwrap();
                                                connection.write(cell);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            },
                            Err(_) => {}
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
