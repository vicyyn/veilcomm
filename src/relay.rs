use crate::{
    payloads::{self, CreatePayload, Payload},
    utils::Connections,
};
use anyhow::{Context, Result};
use log::{error, info, warn};
use openssl::{
    bn::BigNum,
    dh::Dh,
    pkey::Private,
    rsa::{Padding, Rsa},
    symm::{decrypt, Cipher},
};
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    handshakes: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>>,
    keys: Arc<RelayKeys>,
    circuits: Arc<Mutex<HashMap<Uuid, (SocketAddr, Option<SocketAddr>)>>>,
}

impl Relay {
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
            circuits: Arc::new(Mutex::new(HashMap::new())),
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

        let nickname = self.relay_descriptor.nickname.clone();
        let connections = self.connections.clone();
        let keys = self.keys.clone();
        let handshakes = self.handshakes.clone();
        let circuits = self.circuits.clone();

        loop {
            let keys = keys.clone();
            let handshakes = handshakes.clone();
            let connections = connections.clone();
            let nickname = nickname.clone();
            let circuits = circuits.clone();

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
                circuits,
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
        handshakes: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>>,
        circuits: Arc<Mutex<HashMap<Uuid, (SocketAddr, Option<SocketAddr>)>>>,
        nickname: String,
    ) {
        tokio::spawn(async move {
            loop {
                let mut buffer = [0; 10240];
                match read.read(&mut buffer).await {
                    Ok(0) => {}
                    Ok(n) => {
                        info!("{} Read {} bytes", nickname.clone(), n);
                        // try to deserialize, if it fails, then decrypt and try again
                        let payload = if let Ok(payload) =
                            serde_json::from_str::<Payload>(&String::from_utf8_lossy(&buffer[..n]))
                        {
                            payload
                        } else {
                            warn!("Failed to deserialize payload, attempting to decrypt");
                            let handshakes_lock = handshakes.lock().await;
                            let handshake = handshakes_lock.get(&addr).unwrap();
                            let deserialized_payload = decrypt(
                                Cipher::aes_256_ctr(),
                                &handshake[0..32],
                                None,
                                &buffer[..n],
                            )
                            .unwrap();
                            serde_json::from_slice::<Payload>(&deserialized_payload).unwrap()
                        };

                        info!(
                            "Relay {} received payload: {:?}",
                            nickname,
                            payload.get_type()
                        );
                        match payload {
                            Payload::Create(create_payload) => {
                                let onion_skin = create_payload.onion_skin;
                                let mut aes = vec![0; keys.rsa_private.size() as usize];
                                keys.rsa_private
                                    .private_decrypt(
                                        &onion_skin.rsa_encrypted_aes_key,
                                        &mut aes,
                                        Padding::PKCS1,
                                    )
                                    .unwrap();

                                let dh = decrypt(
                                    Cipher::aes_128_ctr(),
                                    &aes[0..16],
                                    None,
                                    &onion_skin.aes_encrypted_dh_key,
                                )
                                .unwrap();

                                let handshake = keys
                                    .dh
                                    .compute_key(&BigNum::from_slice(&dh).unwrap())
                                    .unwrap();

                                info!("Handshake Successful: {}", hex::encode(&handshake[0..16]));

                                let mut handshakes_lock = handshakes.lock().await;
                                handshakes_lock.insert(addr, handshake);
                                drop(handshakes_lock);

                                info!(
                                    "Adding a new circuit with ID: {}",
                                    create_payload.circuit_id
                                );

                                let val = circuits
                                    .lock()
                                    .await
                                    .insert(create_payload.circuit_id, (addr, None));

                                if val.is_some() {
                                    error!("Circuit ID already exists");
                                    continue;
                                }

                                info!("Sending created payload");
                                let created_payload = Payload::Created(payloads::CreatedPayload {
                                    circuit_id: create_payload.circuit_id,
                                    dh_key: keys.dh.public_key().to_vec(),
                                });

                                write
                                    .lock()
                                    .await
                                    .write_all(
                                        serde_json::to_string(&created_payload)
                                            .expect("Failed to serialize JSON")
                                            .as_bytes(),
                                    )
                                    .await
                                    .unwrap();
                                info!("Sent created payload");
                            }
                            Payload::Created(created_payload) => {
                                let circuits_lock = circuits.lock().await;
                                let (prev_relay, next_relay) =
                                    circuits_lock.get(&created_payload.circuit_id).unwrap();
                                if let Some(next_relay) = next_relay {
                                    if next_relay != &addr {
                                        error!("Next relay does not match");
                                        continue;
                                    }
                                }
                                let mut connections_lock = connections.lock().await;
                                let sender = connections_lock.get_mut(&prev_relay).unwrap();
                                info!("Forwarding extended payload to previous relay");
                                let extended_payload =
                                    Payload::Extended(payloads::ExtendedPayload {
                                        circuit_id: created_payload.circuit_id,
                                        dh_key: created_payload.dh_key,
                                    });
                                sender
                                    .lock()
                                    .await
                                    .write_all(
                                        serde_json::to_string(&extended_payload)
                                            .expect("Failed to serialize JSON")
                                            .as_bytes(),
                                    )
                                    .await
                                    .unwrap();
                                info!("Forwarded extended payload to previous relay");
                            }
                            Payload::Extend(extend_payload) => {
                                // forward the extend payload to the next relay
                                let next_relay = extend_payload.address;
                                let mut connections_lock = connections.lock().await;

                                if let Some((prev_relay, mut next_relay)) =
                                    circuits.lock().await.get(&extend_payload.circuit_id)
                                {
                                    if next_relay.is_some() {
                                        error!("Circuit already extended");
                                        continue;
                                    }
                                    info!(
                                        "Updating circuit {} with next relay at address {}",
                                        extend_payload.circuit_id, extend_payload.address
                                    );
                                    next_relay = Some(extend_payload.address);
                                } else {
                                    error!("Circuit ID does not exist");
                                    continue;
                                };

                                // if the next relay is not connected, then try to connect
                                if let Some(next_relay_stream) =
                                    connections_lock.get_mut(&next_relay)
                                {
                                    info!(
                                      "Forwarding create payload to next relay for extension at address {}",
                                      next_relay
                                    );

                                    let create_payload = Payload::Create(CreatePayload {
                                        circuit_id: extend_payload.circuit_id,
                                        onion_skin: extend_payload.onion_skin,
                                    });
                                    next_relay_stream
                                        .lock()
                                        .await
                                        .write_all(
                                            serde_json::to_string(&create_payload)
                                                .unwrap()
                                                .as_bytes(),
                                        )
                                        .await
                                        .unwrap();
                                    info!("Forwarded create payload to next relay");
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
                                                circuits.clone(),
                                                nickname.clone(),
                                            );
                                            connections_lock.insert(next_relay, next_write.clone());
                                            info!(
                                                    "Forwarding create payload to next relay for extension at address {}",
                                                    next_relay
                                                );
                                            let create_payload = Payload::Create(CreatePayload {
                                                circuit_id: extend_payload.circuit_id,
                                                onion_skin: extend_payload.onion_skin,
                                            });
                                            next_write
                                                .lock()
                                                .await
                                                .write_all(
                                                    serde_json::to_string(&create_payload)
                                                        .unwrap()
                                                        .as_bytes(),
                                                )
                                                .await
                                                .unwrap();
                                            info!("Forwarded create payload to next relay");
                                        }
                                        Err(e) => {
                                            error!("Failed to connect to next relay: {}", e);
                                        }
                                    }
                                }
                            }
                            _ => {}
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
