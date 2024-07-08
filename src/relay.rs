use crate::{
    decrypt_buffer_with_aes, encrypt_buffer_with_aes,
    payloads::{self, CreatePayload},
    utils::{get_handshake_from_onion_skin, Connections},
    ConnectedPayload, Payload, PayloadType, RelayCell,
};
use anyhow::{Context, Result};
use log::{error, info, warn};
use openssl::{dh::Dh, pkey::Private, rsa::Rsa};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    sync::Mutex,
};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RelayDescriptor {
    pub address: SocketAddr,
    pub nickname: String,
    pub rsa_public: Vec<u8>,
}

pub struct RelayKeys {
    pub rsa_private: Rsa<Private>,
    pub dh: Dh<Private>,
}

pub struct Relay {
    relay_descriptor: RelayDescriptor,
    connections: Connections,
    handshakes: Arc<Mutex<HashMap<Uuid, Vec<u8>>>>,
    keys: Arc<RelayKeys>,
    circuits_sockets: Arc<Mutex<HashMap<Uuid, SocketAddr>>>,
    // bool is direction, if true then we have to decrypt, if false then we have to encrypt
    circuits_map: Arc<Mutex<HashMap<Uuid, (Uuid, bool)>>>,
    rendezvous_points: Arc<Mutex<HashMap<Uuid, Uuid>>>,
    introduction_points: Arc<Mutex<HashMap<Uuid, Uuid>>>,
    streams: Arc<Mutex<HashMap<Uuid, SocketAddr>>>,
}

impl Relay {
    pub fn get_relay_descriptor(&self) -> RelayDescriptor {
        self.relay_descriptor.clone()
    }

    pub fn new(address: SocketAddr, nickname: String) -> Self {
        info!(
            "Creating new relay with nickname: {} at address: {}",
            nickname, address
        );
        let rsa = openssl::rsa::Rsa::generate(2048).unwrap();
        Self {
            relay_descriptor: RelayDescriptor {
                address,
                nickname,
                rsa_public: rsa.public_key_to_pem().unwrap(),
            },
            connections: Arc::new(Mutex::new(HashMap::new())),
            keys: Arc::new(RelayKeys {
                rsa_private: rsa,
                dh: openssl::dh::Dh::get_2048_256()
                    .unwrap()
                    .generate_key()
                    .unwrap(),
            }),
            handshakes: Arc::new(Mutex::new(HashMap::new())),
            circuits_sockets: Arc::new(Mutex::new(HashMap::new())),
            circuits_map: Arc::new(Mutex::new(HashMap::new())),
            rendezvous_points: Arc::new(Mutex::new(HashMap::new())),
            introduction_points: Arc::new(Mutex::new(HashMap::new())),
            streams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the relay server
    pub async fn start(&self, directory_address: SocketAddr) -> Result<()> {
        info!(
            "Starting the relay server at {}",
            self.relay_descriptor.address
        );

        // Register with the directory server
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/relays",
            directory_address.ip(),
            directory_address.port()
        );

        info!("Registering relay with directory server at {}", url);
        let response = client
            .post(&url)
            .json(&self.relay_descriptor)
            .send()
            .await
            .context("Failed to send registration request")?
            .error_for_status()
            .context("Registration request returned error")?;

        info!("Registration successful with status: {}", response.status());

        let listener = TcpListener::bind(self.relay_descriptor.address).await?;
        info!("TCP server listening on {}", self.relay_descriptor.address);

        loop {
            let keys = self.keys.clone();
            let handshakes = self.handshakes.clone();
            let connections = self.connections.clone();
            let nickname = self.relay_descriptor.nickname.clone();
            let circuits_sockets = self.circuits_sockets.clone();
            let circuits_map = self.circuits_map.clone();
            let rendezvous_points = self.rendezvous_points.clone();
            let introduction_points = self.introduction_points.clone();
            let streams = self.streams.clone();

            let (stream, addr) = listener.accept().await?;
            info!("Accepted connection from {}", addr);
            let (read, write) = stream.into_split();
            let write = Arc::new(Mutex::new(write));
            connections.lock().await.insert(addr, write.clone());
            Self::handle_read(
                read,
                write,
                addr,
                connections,
                keys,
                handshakes,
                circuits_sockets,
                circuits_map,
                rendezvous_points,
                introduction_points,
                streams,
                nickname,
            );
        }
    }

    pub fn handle_read(
        mut read: OwnedReadHalf,
        write: Arc<Mutex<OwnedWriteHalf>>,
        addr: SocketAddr,
        connections: Connections,
        keys: Arc<RelayKeys>,
        handshakes: Arc<Mutex<HashMap<Uuid, Vec<u8>>>>,
        circuits_sockets: Arc<Mutex<HashMap<Uuid, SocketAddr>>>,
        circuits_map: Arc<Mutex<HashMap<Uuid, (Uuid, bool)>>>,
        rendezvous_points: Arc<Mutex<HashMap<Uuid, Uuid>>>,
        introduction_points: Arc<Mutex<HashMap<Uuid, Uuid>>>,
        streams: Arc<Mutex<HashMap<Uuid, SocketAddr>>>,
        nickname: String,
    ) {
        tokio::spawn(async move {
            loop {
                let mut buffer = [0; 50240];
                match read.read(&mut buffer).await {
                    Ok(0) => {}
                    Ok(n) => {
                        info!("{} Read {} bytes", nickname.clone(), n);
                        // deserialize relay cell
                        let relay_cell = if let Ok(relay_cell) =
                            serde_json::from_slice::<RelayCell>(&buffer[0..n])
                        {
                            relay_cell
                        } else {
                            error!("Failed to deserialize relay cell coming from {}", addr);
                            continue;
                        };
                        info!("{} Received relay cell", nickname.clone());

                        // check if there's a next relay
                        let mut circuits_map_lock = circuits_map.lock().await;
                        let mut handshakes_lock = handshakes.lock().await;
                        let mut circuits_sockets_lock = circuits_sockets.lock().await;
                        let mut connections_lock = connections.lock().await;
                        let mut rendezvous_points_lock = rendezvous_points.lock().await;
                        let mut introduction_points_lock = introduction_points.lock().await;
                        let mut streams_lock = streams.lock().await;

                        if let Some((next_circuit_id, direction)) =
                            circuits_map_lock.get(&relay_cell.circuit_id)
                        {
                            if *direction {
                                // decrypt with handshake then forward to next relay
                                let handshake =
                                    handshakes_lock.get(&relay_cell.circuit_id).unwrap();
                                let decrypted_payload =
                                    decrypt_buffer_with_aes(&handshake[0..32], &relay_cell.payload)
                                        .unwrap();
                                if let Ok(payload) =
                                    serde_json::from_slice::<Payload>(&decrypted_payload)
                                {
                                    if payload.get_type() == PayloadType::Data {
                                        info!("{} forwarding data payload", nickname.clone());
                                        let socket =
                                            circuits_sockets_lock.get(next_circuit_id).unwrap();
                                        let sender = connections_lock.get_mut(&socket).unwrap();
                                        let handshake = handshakes_lock
                                            .get(next_circuit_id)
                                            .expect("Handshake not found");
                                        let encrypted_payload =
                                            encrypt_buffer_with_aes(&handshake, &decrypted_payload)
                                                .unwrap();
                                        let relay_cell = RelayCell {
                                            circuit_id: *next_circuit_id,
                                            payload: encrypted_payload,
                                        };
                                        sender
                                            .lock()
                                            .await
                                            .write_all(&serde_json::to_vec(&relay_cell).unwrap())
                                            .await
                                            .unwrap();
                                        info!("{} forwarded data payload", nickname.clone());
                                    } else {
                                        error!(
                                            "{} expected data payload, got {:?} payload",
                                            nickname.clone(),
                                            payload.get_type()
                                        );
                                    }
                                } else {
                                    let relay_cell = RelayCell {
                                        circuit_id: *next_circuit_id,
                                        payload: decrypted_payload,
                                    };
                                    let next_socket =
                                        circuits_sockets_lock.get(next_circuit_id).unwrap();
                                    let connection = connections_lock.get(next_socket).unwrap();
                                    info!(
                                        "{} forwarding relay cell to next relay",
                                        nickname.clone()
                                    );
                                    connection
                                        .lock()
                                        .await
                                        .write_all(&serde_json::to_vec(&relay_cell).unwrap())
                                        .await
                                        .unwrap();
                                }
                                continue;
                            }
                        };

                        // get the payload
                        let payload = if let Some(handshake) =
                            handshakes_lock.get(&relay_cell.circuit_id)
                        {
                            let decrypted_payload =
                                decrypt_buffer_with_aes(&handshake[0..32], &relay_cell.payload)
                                    .unwrap();
                            info!(
                                "Decrypted payload with handshake for circuit {}",
                                relay_cell.circuit_id
                            );
                            serde_json::from_slice::<Payload>(&decrypted_payload).unwrap()
                        } else {
                            info!("No handshake found for circuit {}", relay_cell.circuit_id);
                            if let Ok(payload) =
                                serde_json::from_slice::<Payload>(&relay_cell.payload)
                            {
                                payload
                            } else {
                                if let Some((next_circuit_id, direction)) =
                                    circuits_map_lock.get(&relay_cell.circuit_id)
                                {
                                    if !*direction {
                                        let socket =
                                            circuits_sockets_lock.get(next_circuit_id).unwrap();
                                        let sender = connections_lock.get_mut(&socket).unwrap();
                                        info!(
                                            "Forwarding payload back to circuit {}",
                                            next_circuit_id
                                        );
                                        let handshake =
                                            handshakes_lock.get(&next_circuit_id).unwrap();
                                        let encrypted_payload = encrypt_buffer_with_aes(
                                            &handshake,
                                            &relay_cell.payload,
                                        )
                                        .unwrap();
                                        let relay_cell = RelayCell {
                                            circuit_id: *next_circuit_id,
                                            payload: encrypted_payload,
                                        };
                                        sender
                                            .lock()
                                            .await
                                            .write_all(&serde_json::to_vec(&relay_cell).unwrap())
                                            .await
                                            .unwrap();
                                        info!("Forwarded payload to previous relay");
                                    } else {
                                        error!("direction is wrong, expected false, got true for circuit {} coming from {}",
                                            relay_cell.circuit_id,
                                            addr
                                        );
                                    }
                                    continue;
                                } else {
                                    error!(
                                        "no circuit found for circuit {} coming from {}",
                                        relay_cell.circuit_id, addr
                                    );
                                    continue;
                                }
                            }
                        };

                        info!(
                            "{} Received payload: {:?}",
                            nickname.clone(),
                            payload.get_type()
                        );

                        match payload {
                            Payload::Create(create_payload) => {
                                let handshake = get_handshake_from_onion_skin(
                                    create_payload.onion_skin,
                                    &keys.dh,
                                    &keys.rsa_private,
                                );

                                handshakes_lock.insert(relay_cell.circuit_id.clone(), handshake);
                                drop(handshakes_lock);
                                info!(
                                    "Adding a new circuit with ID: {}",
                                    relay_cell.circuit_id.clone()
                                );

                                if circuits_sockets_lock
                                    .insert(relay_cell.circuit_id, addr)
                                    .is_some()
                                {
                                    error!("Circuit ID already exists");
                                    continue;
                                }

                                info!("Sending created payload");
                                let created_payload = Payload::Created(payloads::CreatedPayload {
                                    dh_key: keys.dh.public_key().to_vec(),
                                });
                                let relay_cell = RelayCell {
                                    circuit_id: relay_cell.circuit_id,
                                    payload: serde_json::to_vec(&created_payload).unwrap(),
                                };

                                write
                                    .lock()
                                    .await
                                    .write_all(
                                        &serde_json::to_vec(&relay_cell)
                                            .expect("Failed to serialize JSON"),
                                    )
                                    .await
                                    .unwrap();
                                info!("Sent created payload");
                            }
                            Payload::Created(created_payload) => {
                                if let Some((next_circuit_id, direction)) =
                                    circuits_map_lock.get(&relay_cell.circuit_id)
                                {
                                    if !*direction {
                                        let socket =
                                            circuits_sockets_lock.get(next_circuit_id).unwrap();
                                        let sender = connections_lock.get_mut(&socket).unwrap();
                                        info!(
                                            "Forwarding extended payload back to circuit {}",
                                            next_circuit_id
                                        );
                                        let extended_payload =
                                            Payload::Extended(payloads::ExtendedPayload {
                                                address: addr,
                                                dh_key: created_payload.dh_key,
                                            });
                                        let handshake =
                                            handshakes_lock.get(&next_circuit_id).unwrap();
                                        let encrypted_payload = encrypt_buffer_with_aes(
                                            &handshake,
                                            &serde_json::to_vec(&extended_payload).unwrap(),
                                        )
                                        .unwrap();
                                        let relay_cell = RelayCell {
                                            circuit_id: *next_circuit_id,
                                            payload: encrypted_payload,
                                        };
                                        sender
                                            .lock()
                                            .await
                                            .write_all(&serde_json::to_vec(&relay_cell).unwrap())
                                            .await
                                            .unwrap();
                                        info!("Forwarded payload to previous relay");
                                    } else {
                                        error!("direction is wrong, expected false, got true for circuit {} coming from {}",
                                            relay_cell.circuit_id,
                                            addr
                                        );
                                    }
                                }
                            }
                            Payload::Extend(extend_payload) => {
                                let next_relay = extend_payload.address;
                                // Check if the circuit is already extended
                                if let Some(_) = circuits_map_lock.get(&relay_cell.circuit_id) {
                                    error!("Circuit already extended");
                                    continue;
                                }
                                info!("Extending circuit with ID: {}", relay_cell.circuit_id);
                                let new_circuit_id = Uuid::new_v4();
                                circuits_map_lock
                                    .insert(relay_cell.circuit_id, (new_circuit_id, true));
                                circuits_map_lock
                                    .insert(new_circuit_id, (relay_cell.circuit_id, false));
                                circuits_sockets_lock.insert(new_circuit_id, next_relay);

                                // forward the extend payload to the next relay as create payload
                                let create_payload = Payload::Create(CreatePayload {
                                    onion_skin: extend_payload.onion_skin,
                                });
                                let relay_cell = RelayCell {
                                    circuit_id: new_circuit_id,
                                    payload: serde_json::to_vec(&create_payload)
                                        .expect("Failed to serialize JSON"),
                                };

                                // connect to the next relay
                                if let Some(next_relay_stream) =
                                    connections_lock.get_mut(&next_relay)
                                {
                                    next_relay_stream
                                        .lock()
                                        .await
                                        .write_all(
                                            serde_json::to_vec(&relay_cell)
                                                .expect("Failed to serialize JSON")
                                                .as_slice(),
                                        )
                                        .await
                                        .unwrap();
                                } else {
                                    warn!("Next relay not connected, attempting to connect");
                                    match tokio::net::TcpStream::connect(next_relay).await {
                                        Ok(next_relay_stream) => {
                                            info!(
                                                "Connected to next relay at address {}",
                                                next_relay
                                            );
                                            let new_addr = next_relay_stream.local_addr().unwrap();
                                            let (next_read, next_write) =
                                                next_relay_stream.into_split();
                                            let next_write = Arc::new(Mutex::new(next_write));
                                            Self::handle_read(
                                                next_read,
                                                next_write.clone(),
                                                new_addr,
                                                connections.clone(),
                                                keys.clone(),
                                                handshakes.clone(),
                                                circuits_sockets.clone(),
                                                circuits_map.clone(),
                                                rendezvous_points.clone(),
                                                introduction_points.clone(),
                                                streams.clone(),
                                                nickname.clone(),
                                            );
                                            connections_lock.insert(next_relay, next_write.clone());
                                            next_write
                                                .lock()
                                                .await
                                                .write_all(
                                                    &serde_json::to_vec(&relay_cell).unwrap(),
                                                )
                                                .await
                                                .unwrap();
                                            info!("Forwarded create payload to next relay");
                                        }
                                        _ => {
                                            error!(
                                                "Failed to connect to next relay {}",
                                                next_relay
                                            );
                                            continue;
                                        }
                                    }
                                };
                            }
                            Payload::EstablishRendezvous(establish_rendezvous) => {
                                let rendezvous_cookie = establish_rendezvous.rendezvous_cookie;
                                rendezvous_points_lock
                                    .insert(rendezvous_cookie, relay_cell.circuit_id);
                                let established_rendezvous_payload = Payload::EstablishedRendezvous(
                                    payloads::EstablishedRendezvousPayload {},
                                );
                                let handshake = handshakes_lock
                                    .get(&relay_cell.circuit_id)
                                    .expect("Handshake not found");
                                let encrypted_payload = encrypt_buffer_with_aes(
                                    &handshake[0..32],
                                    &serde_json::to_vec(&established_rendezvous_payload)
                                        .expect("Failed to serialize JSON"),
                                )
                                .unwrap();
                                let relay_cell = RelayCell {
                                    circuit_id: relay_cell.circuit_id,
                                    payload: encrypted_payload,
                                };
                                write
                                    .lock()
                                    .await
                                    .write_all(
                                        &serde_json::to_vec(&relay_cell)
                                            .expect("Failed to serialize JSON"),
                                    )
                                    .await
                                    .unwrap();
                            }
                            Payload::EstablishIntroduction(establish_introduction) => {
                                let introduction_id = establish_introduction.introduction_id;
                                introduction_points_lock
                                    .insert(introduction_id, relay_cell.circuit_id);
                                let established_introduction_payload =
                                    Payload::EstablishedIntroduction(
                                        payloads::EstablishedIntroductionPayload {},
                                    );
                                let handshake = handshakes_lock
                                    .get(&relay_cell.circuit_id)
                                    .expect("Handshake not found");
                                let encrypted_payload = encrypt_buffer_with_aes(
                                    &handshake,
                                    &serde_json::to_vec(&established_introduction_payload)
                                        .expect("Failed to serialize JSON"),
                                )
                                .unwrap();
                                let relay_cell = RelayCell {
                                    circuit_id: relay_cell.circuit_id,
                                    payload: encrypted_payload,
                                };
                                write
                                    .lock()
                                    .await
                                    .write_all(
                                        &serde_json::to_vec(&relay_cell)
                                            .expect("Failed to serialize JSON"),
                                    )
                                    .await
                                    .unwrap();
                            }
                            Payload::Begin(begin_payload) => {
                                let begin_relay = begin_payload.relay_descriptor.address;
                                let connected_payload = Payload::Connected(ConnectedPayload {});
                                let handshake =
                                    handshakes_lock.get(&relay_cell.circuit_id).unwrap();
                                let encrypted_payload = encrypt_buffer_with_aes(
                                    &handshake,
                                    &serde_json::to_vec(&connected_payload).unwrap(),
                                )
                                .unwrap();
                                let begin_relay_cell = RelayCell {
                                    circuit_id: relay_cell.circuit_id,
                                    payload: encrypted_payload,
                                };
                                if let Some(_) = connections_lock.get_mut(&begin_relay) {
                                    streams_lock.insert(begin_payload.stream_id, begin_relay);
                                    write
                                        .lock()
                                        .await
                                        .write_all(
                                            serde_json::to_vec(&begin_relay_cell)
                                                .expect("Failed to serialize JSON")
                                                .as_slice(),
                                        )
                                        .await
                                        .unwrap();
                                } else {
                                    warn!("Begin relay not connected, attempting to connect");
                                    match tokio::net::TcpStream::connect(begin_relay).await {
                                        Ok(next_relay_stream) => {
                                            warn!(
                                                "Connected to begin relay at address {}",
                                                begin_relay
                                            );
                                            streams_lock
                                                .insert(begin_payload.stream_id, begin_relay);
                                            let new_addr = next_relay_stream.local_addr().unwrap();
                                            let (next_read, next_write) =
                                                next_relay_stream.into_split();
                                            let next_write = Arc::new(Mutex::new(next_write));
                                            Self::handle_read(
                                                next_read,
                                                next_write.clone(),
                                                new_addr,
                                                connections.clone(),
                                                keys.clone(),
                                                handshakes.clone(),
                                                circuits_sockets.clone(),
                                                circuits_map.clone(),
                                                rendezvous_points.clone(),
                                                introduction_points.clone(),
                                                streams.clone(),
                                                nickname.clone(),
                                            );
                                            connections_lock.insert(begin_relay, next_write);
                                            write
                                                .lock()
                                                .await
                                                .write_all(
                                                    &serde_json::to_vec(&begin_relay_cell).unwrap(),
                                                )
                                                .await
                                                .unwrap();
                                        }
                                        _ => {
                                            error!(
                                                "Failed to connect to begin relay {}",
                                                begin_relay
                                            );
                                            continue;
                                        }
                                    }
                                };
                            }
                            Payload::Introduce1(introduce1_payload) => {
                                // verify that introduction id matches and that the stream exists
                                let stream_id = introduce1_payload.stream_id;
                                let introduction_id = introduce1_payload.introduction_id;

                                if let Some(socket) = streams_lock.get(&stream_id) {
                                    info!("{} Stream found", nickname.clone());
                                    let connection = connections_lock.get_mut(socket).unwrap();
                                    let introduce1_payload =
                                        Payload::Introduce1(payloads::Introduce1Payload {
                                            stream_id,
                                            introduction_id,
                                            rendezvous_point_descriptor: introduce1_payload
                                                .rendezvous_point_descriptor,
                                            rendezvous_cookie: introduce1_payload.rendezvous_cookie,
                                            onion_skin: introduce1_payload.onion_skin,
                                        });

                                    let relay_cell = RelayCell {
                                        circuit_id: relay_cell.circuit_id,
                                        payload: serde_json::to_vec(&introduce1_payload).unwrap(),
                                    };
                                    connection
                                        .lock()
                                        .await
                                        .write_all(&serde_json::to_vec(&relay_cell).unwrap())
                                        .await
                                        .unwrap();
                                    info!(
                                        "{} Sent introduce1 payload to stream {}",
                                        nickname.clone(),
                                        stream_id
                                    );

                                    info!("{} Sending introduce ack payload", nickname.clone());

                                    let introduce_ack_payload =
                                        Payload::IntroduceAck(payloads::IntroduceAckPayload {});
                                    let handshake = handshakes_lock
                                        .get(&relay_cell.circuit_id)
                                        .expect("Handshake not found");
                                    let encrypted_payload = encrypt_buffer_with_aes(
                                        &handshake,
                                        &serde_json::to_vec(&introduce_ack_payload).unwrap(),
                                    )
                                    .unwrap();
                                    let relay_cell = RelayCell {
                                        circuit_id: relay_cell.circuit_id,
                                        payload: encrypted_payload,
                                    };

                                    // respond with introduce ack
                                    write
                                        .lock()
                                        .await
                                        .write_all(&serde_json::to_vec(&relay_cell).unwrap())
                                        .await
                                        .unwrap();

                                    info!("{} Sent introduce ack payload", nickname.clone());
                                } else {
                                    warn!("Stream not found");
                                    if let Some(introduction_circuit_id) =
                                        introduction_points_lock.get(&introduction_id)
                                    {
                                        info!("{} Introduction point found", nickname.clone());
                                        let introduction_socket = circuits_sockets_lock
                                            .get(introduction_circuit_id)
                                            .expect("Introduction point not found");
                                        info!(
                                            "{} Sending introduce2 payload to introduction socket {}",
                                            nickname.clone(),
                                            introduction_socket
                                        );
                                        let introduce2_payload =
                                            Payload::Introduce2(payloads::Introduce2Payload {
                                                rendezvous_point_descriptor: introduce1_payload
                                                    .rendezvous_point_descriptor,
                                                rendezvous_cookie: introduce1_payload
                                                    .rendezvous_cookie,
                                                onion_skin: introduce1_payload.onion_skin,
                                            });
                                        let handshake = handshakes_lock
                                            .get(&introduction_circuit_id)
                                            .expect("Handshake not found");
                                        let introduce2_payload = encrypt_buffer_with_aes(
                                            &handshake,
                                            &serde_json::to_vec(&introduce2_payload).unwrap(),
                                        )
                                        .unwrap();
                                        let relay_cell = RelayCell {
                                            circuit_id: *introduction_circuit_id,
                                            payload: introduce2_payload,
                                        };
                                        let connection =
                                            connections_lock.get_mut(introduction_socket).unwrap();
                                        connection
                                            .lock()
                                            .await
                                            .write_all(&serde_json::to_vec(&relay_cell).unwrap())
                                            .await
                                            .unwrap();
                                    } else {
                                        error!("{} Introduction point not found", nickname.clone());
                                        continue;
                                    }
                                }
                            }
                            Payload::Rendezvous1(rendezvous1_payload) => {
                                // connect the two circuits together
                                if let Some(original_circuit_id) = rendezvous_points_lock
                                    .get(&rendezvous1_payload.rendezvous_cookie)
                                {
                                    circuits_map_lock.insert(
                                        relay_cell.circuit_id,
                                        (*original_circuit_id, false),
                                    );
                                    circuits_map_lock.insert(
                                        *original_circuit_id,
                                        (relay_cell.circuit_id, true),
                                    );
                                    let rendezvous2_payload =
                                        Payload::Rendezvous2(payloads::Rendezvous2Payload {
                                            rendezvous_cookie: rendezvous1_payload
                                                .rendezvous_cookie,
                                            dh_key: rendezvous1_payload.dh_key,
                                        });
                                    let handshake = handshakes_lock
                                        .get(original_circuit_id)
                                        .expect("Handshake not found");
                                    let encrypted_payload = encrypt_buffer_with_aes(
                                        &handshake,
                                        &serde_json::to_vec(&rendezvous2_payload).unwrap(),
                                    )
                                    .unwrap();
                                    let relay_cell = RelayCell {
                                        circuit_id: *original_circuit_id,
                                        payload: encrypted_payload,
                                    };
                                    let socket = circuits_sockets_lock
                                        .get(original_circuit_id)
                                        .expect("Original circuit not found");
                                    let connection = connections_lock.get_mut(socket).unwrap();
                                    connection
                                        .lock()
                                        .await
                                        .write_all(&serde_json::to_vec(&relay_cell).unwrap())
                                        .await
                                        .unwrap();
                                } else {
                                    error!("Rendezvous point not found");
                                    continue;
                                }
                            }
                            Payload::Data(_) => {}
                            _ => {
                                error!("Unhandled payload type");
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from socket: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
