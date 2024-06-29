use crate::payloads::{CreatePayload, EstablishIntroPayload, ExtendPayload, OnionSkin};
use crate::relay::RelayDescriptor;
use crate::relay_cell::RelayCell;
use crate::utils::{generate_random_aes_key, Connections};
use crate::{deserialize_payload_with_aes_keys, serialize_payload_with_aes_keys, Event, Payload};
use anyhow::Result;
use log::{error, info};
use openssl::bn::BigNum;
use openssl::dh::Dh;
use openssl::pkey::Private;
use openssl::rsa::Rsa;
use openssl::symm::{encrypt, Cipher};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserDescriptor {
    pub nickname: String,
    pub publickey: Vec<u8>,
    pub introduction_points: Vec<(Uuid, SocketAddr)>,
}

pub struct UserKeys {
    pub rsa_private: Rsa<Private>,
    pub dh: Dh<Private>,
}

pub struct User {
    user_descriptor: UserDescriptor,
    fetched_relays: Mutex<Vec<RelayDescriptor>>,
    connections: Connections,
    keys: Arc<UserKeys>,
    handshakes: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>>,
    pub events_receiver: Arc<Mutex<Receiver<Event>>>,
    events_sender: Arc<Mutex<Sender<Event>>>,
    circuits: Arc<Mutex<HashMap<Uuid, Vec<SocketAddr>>>>,
}

impl User {
    pub fn new(nickname: String, publickey: Vec<u8>) -> Self {
        info!("Creating new user with publickey {:?}", publickey);
        let (events_sender, events_receiver) = tokio::sync::mpsc::channel(100);
        Self {
            user_descriptor: UserDescriptor {
                nickname,
                publickey,
                introduction_points: Vec::new(),
            },
            connections: Arc::new(Mutex::new(HashMap::new())),
            fetched_relays: Mutex::new(Vec::new()),
            keys: Arc::new(UserKeys {
                rsa_private: Rsa::generate(2048).unwrap(),
                dh: Dh::get_2048_256().unwrap().generate_key().unwrap(),
            }),
            handshakes: Arc::new(Mutex::new(HashMap::new())),
            events_receiver: Arc::new(Mutex::new(events_receiver)),
            events_sender: Arc::new(Mutex::new(events_sender)),
            circuits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self, directory_address: SocketAddr) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/users",
            directory_address.ip(),
            directory_address.port()
        );
        info!(
            "{} Registering user with directory server at URL: {}",
            self.user_descriptor.nickname, url
        );
        match client.post(&url).json(&self.user_descriptor).send().await {
            Ok(response) => {
                response.error_for_status_ref()?;
                info!(
                    "{} registered successfully with status: {}",
                    self.user_descriptor.nickname,
                    response.status()
                );
            }
            Err(e) => {
                error!(
                    "Failed to register user {}: {}",
                    self.user_descriptor.nickname, e
                );
                return Err(e.into());
            }
        }
        Ok(())
    }

    /// Fetch all relays from the directory server
    pub async fn fetch_relays(&self, directory_address: SocketAddr) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/relays",
            directory_address.ip(),
            directory_address.port()
        );
        info!(
            "{} fetching relays from directory server at URL: {}",
            self.user_descriptor.nickname, url
        );
        let response = client.get(&url).send().await?;
        let relays_fetched: Vec<RelayDescriptor> = response.json().await?;
        info!(
            "{} fetched {:?} relays",
            self.user_descriptor.nickname,
            relays_fetched.len()
        );
        let mut relays = self.fetched_relays.lock().await;
        relays.clear();
        relays.extend(relays_fetched);
        Ok(())
    }

    /// Connect to a relay server
    pub async fn connect_to_relay(&self, relay_discriptor: RelayDescriptor) -> Result<()> {
        info!(
            "{} connecting to relay {}",
            self.user_descriptor.nickname, relay_discriptor.nickname
        );
        let stream = TcpStream::connect(relay_discriptor.address).await?;
        info!(
            "{} connected to relay {}",
            self.user_descriptor.nickname, relay_discriptor.nickname
        );
        let (read, write) = stream.into_split();
        let write = Arc::new(Mutex::new(write));
        self.connections
            .lock()
            .await
            .insert(relay_discriptor.address, write);
        Self::handle_read(
            read,
            self.keys.clone(),
            self.events_sender.clone(),
            self.circuits.clone(),
            self.handshakes.clone(),
            self.user_descriptor.nickname.clone(),
            relay_discriptor.nickname.clone(),
        );
        Ok(())
    }

    pub fn handle_read(
        mut read: OwnedReadHalf,
        keys: Arc<UserKeys>,
        events_sender: Arc<Mutex<Sender<Event>>>,
        circuits: Arc<Mutex<HashMap<Uuid, Vec<SocketAddr>>>>,
        handshakes: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>>,
        nickname: String,
        relay_nickname: String,
    ) {
        let relay_address = read.peer_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let keys = keys.clone();
                let mut buffer = [0; 10240];
                match read.read(&mut buffer).await {
                    Ok(0) => {}
                    Ok(n) => {
                        let relay_cell: RelayCell = serde_json::from_slice(&buffer[..n]).unwrap();
                        info!(
                            "{} received a relay cell from {:?} for circuit id {}",
                            nickname, relay_nickname, relay_cell.circuit_id
                        );
                        let mut circuits_lock = circuits.lock().await;
                        let mut handshakes_lock = handshakes.lock().await;
                        let payload = if let Some(circuit) =
                            circuits_lock.get(&relay_cell.circuit_id)
                        {
                            let mut vec_handshakes = vec![];
                            for relay in circuit {
                                vec_handshakes.push(handshakes_lock.get(relay).unwrap().clone());
                            }
                            deserialize_payload_with_aes_keys(vec_handshakes, &relay_cell.payload)
                                .unwrap()
                        } else {
                            serde_json::from_slice(&relay_cell.payload).unwrap()
                        };
                        info!("{} received payload: {:?}", nickname, payload.get_type());
                        let payload_type = payload.get_type();
                        match payload {
                            Payload::Created(created_payload) => {
                                let handshake = keys
                                    .dh
                                    .compute_key(
                                        &BigNum::from_slice(&created_payload.dh_key).unwrap(),
                                    )
                                    .unwrap();
                                info!("Handshake Successful: {}", hex::encode(&handshake[0..32]));
                                handshakes_lock.insert(relay_address, handshake);
                                circuits_lock.insert(relay_cell.circuit_id, vec![relay_address]);
                                info!(
                                    "{} added a new circuit with ID: {}",
                                    nickname, relay_cell.circuit_id
                                );
                            }
                            Payload::Extended(extended_payload) => {
                                let handshake = keys
                                    .dh
                                    .compute_key(
                                        &BigNum::from_slice(&extended_payload.dh_key).unwrap(),
                                    )
                                    .unwrap();
                                info!("Handshake Successful: {}", hex::encode(&handshake[0..16]));
                                handshakes_lock.insert(extended_payload.address, handshake);
                                circuits_lock
                                    .get_mut(&relay_cell.circuit_id)
                                    .unwrap()
                                    .push(extended_payload.address);
                                info!(
                                    "{} extended circuit with ID: {} to relay at address: {}",
                                    nickname, relay_cell.circuit_id, extended_payload.address
                                );
                            }
                            _ => {}
                        }
                        events_sender
                            .lock()
                            .await
                            .send(Event(payload_type, relay_address))
                            .await
                            .unwrap();
                    }
                    Err(e) => {
                        error!("Failed to read from socket: {}", e);
                        break;
                    }
                }
            }
        });
    }

    pub async fn send_create_to_relay(
        &self,
        relay_address: SocketAddr,
        circuit_id: Uuid,
    ) -> Result<()> {
        let fetched_relays_lock = self.fetched_relays.lock().await;
        let relay_descriptor = fetched_relays_lock
            .iter()
            .find(|r| r.address == relay_address)
            .unwrap();
        info!(
            "{} sending CREATE payload to: {}",
            self.user_descriptor.nickname, relay_descriptor.nickname
        );
        let rsa_public = Rsa::public_key_from_pem(&relay_descriptor.rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = self.keys.dh.public_key().to_vec();
        let aes = generate_random_aes_key();
        let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
        let create_payload = Payload::Create(CreatePayload { onion_skin });
        let relay_cell = RelayCell {
            circuit_id,
            payload: serde_json::to_vec(&create_payload)?,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        info!(
            "{} sent CREATE payload to: {}",
            self.user_descriptor.nickname, relay_descriptor.nickname
        );
        Ok(())
    }

    pub async fn send_extend_to_relay(
        &self,
        relay_address: SocketAddr,
        relay_address_2: SocketAddr,
        circuit_id: Uuid,
    ) -> Result<()> {
        info!(
            "Extending circuit from relay at address: {} to relay at address: {}",
            relay_address, relay_address_2
        );
        let fetched_relays_lock = self.fetched_relays.lock().await;
        let relay_descriptor = fetched_relays_lock
            .iter()
            .find(|r| r.address == relay_address_2)
            .unwrap()
            .clone();
        drop(fetched_relays_lock);
        let rsa_public = Rsa::public_key_from_pem(&relay_descriptor.rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = self.keys.dh.public_key().to_vec();
        let aes = generate_random_aes_key();
        let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
        let extend_payload = Payload::Extend(ExtendPayload {
            onion_skin,
            address: relay_address_2,
        });
        let circuits_lock = self.circuits.lock().await;
        let circuit = circuits_lock.get(&circuit_id).unwrap();
        let mut handshakes = vec![];
        let handshakes_lock = self.handshakes.lock().await;
        for relay in circuit {
            handshakes.push(handshakes_lock.get(relay).unwrap().clone());
        }
        drop(circuits_lock);
        drop(handshakes_lock);
        let encrypted_payload =
            serialize_payload_with_aes_keys(handshakes, &extend_payload).unwrap();
        let relay_cell = RelayCell {
            circuit_id,
            payload: encrypted_payload,
        };
        info!(
            "Sending EXTEND payload to relay {}",
            relay_descriptor.nickname
        );
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell).unwrap())
            .await?;
        info!("Sent EXTEND payload to relay {}", relay_descriptor.nickname);
        Ok(())
    }

    pub async fn send_establish_intro_to_relay(
        &self,
        relay_address: SocketAddr,
        introduction_id: Uuid,
        circuit_id: Uuid,
    ) -> Result<()> {
        info!(
            "Sending ESTABLISH_INTRO to relay at address: {}",
            relay_address
        );
        let establish_intro_payload =
            Payload::EstablishIntro(EstablishIntroPayload { introduction_id });
        let handshakes_lock = self.handshakes.lock().await;
        let mut establish_intro_serialized_encrypted =
            serde_json::to_vec(&establish_intro_payload)?;
        for relay in self
            .circuits
            .lock()
            .await
            .get(&circuit_id)
            .unwrap()
            .iter()
            .rev()
        {
            let handshake = handshakes_lock.get(&relay).unwrap();
            establish_intro_serialized_encrypted = encrypt(
                Cipher::aes_256_ctr(),
                &handshake[0..32],
                None,
                &establish_intro_serialized_encrypted.clone(),
            )?;
        }
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&establish_intro_payload)?)
            .await?;
        info!(
            "Sent ESTABLISH_INTRO payload to relay at address: {}",
            relay_address
        );
        Ok(())
    }

    pub async fn listen_for_event(&self, event: Event) -> Result<()> {
        loop {
            let received_event = self.events_receiver.lock().await.recv().await.unwrap();
            if received_event == event {
                return Ok(());
            }
        }
    }
}
