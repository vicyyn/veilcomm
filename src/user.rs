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
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::payloads::{CreatePayload, Event, ExtendPayload, OnionSkin, Payload};
use crate::relay::RelayDescriptor;
use crate::utils::{generate_random_aes_key, Connections};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserDescriptor {
    pub publickey: Vec<u8>,
    pub introduction_points: Vec<SocketAddr>,
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
    circuits: Arc<Mutex<HashMap<Uuid, SocketAddr>>>,
}

impl User {
    pub fn new(publickey: Vec<u8>, introduction_points: Vec<SocketAddr>) -> Self {
        info!("Creating new user with publickey {:?}", publickey);
        let (events_sender, events_receiver) = tokio::sync::mpsc::channel(100);
        Self {
            user_descriptor: UserDescriptor {
                publickey,
                introduction_points,
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
        info!("Registering user with directory server at URL: {}", url);
        match client.post(&url).json(&self.user_descriptor).send().await {
            Ok(response) => {
                response.error_for_status_ref()?;
                info!(
                    "User registered successfully with status: {}",
                    response.status()
                );
            }
            Err(e) => {
                error!("Failed to register user: {}", e);
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
        info!("Fetching relays from directory server at URL: {}", url);
        let response = client.get(&url).send().await?;
        let relays_fetched: Vec<RelayDescriptor> = response.json().await?;
        info!("Fetched {:?} relays", relays_fetched.len());
        let mut relays = self.fetched_relays.lock().await;
        relays.clear();
        relays.extend(relays_fetched);
        Ok(())
    }

    /// Connect to a relay server
    pub async fn connect_to_relay(&self, relay_address: SocketAddr) -> Result<()> {
        info!("Connecting to relay at address: {}", relay_address);
        let stream = TcpStream::connect(relay_address).await?;
        let (mut read, write) = stream.into_split();
        let write = Arc::new(Mutex::new(write));
        let handshakes = self.handshakes.clone();
        let events_sender = self.events_sender.clone();
        self.connections
            .lock()
            .await
            .insert(relay_address, write.clone());
        info!("Connected to relay at address: {}", relay_address);
        let keys = self.keys.clone();
        tokio::spawn(async move {
            loop {
                let keys = keys.clone();
                let mut buffer = [0; 10240];
                match read.read(&mut buffer).await {
                    Ok(0) => {}
                    Ok(n) => {
                        let payload: Payload = serde_json::from_slice(&buffer[..n]).unwrap();
                        info!("Received payload: {:?}", payload.get_type());
                        events_sender
                            .lock()
                            .await
                            .send(Event(payload.get_type(), relay_address))
                            .await
                            .unwrap();
                        match payload {
                            Payload::Created(created_payload) => {
                                let handshake = keys
                                    .dh
                                    .compute_key(
                                        &BigNum::from_slice(&created_payload.dh_key).unwrap(),
                                    )
                                    .unwrap();
                                info!("Handshake Successful: {}", hex::encode(&handshake[0..16]));
                                let mut handshakes_lock = handshakes.lock().await;
                                handshakes_lock.insert(relay_address, handshake);
                            }
                            Payload::Extended(extended_payload) => {
                                let handshake = keys
                                    .dh
                                    .compute_key(
                                        &BigNum::from_slice(&extended_payload.dh_key).unwrap(),
                                    )
                                    .unwrap();
                                info!("Handshake Successful: {}", hex::encode(&handshake[0..16]));
                                let mut handshakes_lock = handshakes.lock().await;
                                handshakes_lock.insert(relay_address, handshake);
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
        Ok(())
    }

    pub async fn send_create_to_relay(&self, relay_address: SocketAddr) -> Result<()> {
        info!("Sending CREATE to relay at address: {}", relay_address);
        let fetched_relays_lock = self.fetched_relays.lock().await;
        let relay = fetched_relays_lock
            .iter()
            .find(|r| r.address == relay_address)
            .unwrap();
        let rsa_public = Rsa::public_key_from_pem(&relay.rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = self.keys.dh.public_key().to_vec();
        let aes = generate_random_aes_key();
        let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
        let circuit_id = Uuid::new_v4();
        let create_payload = Payload::Create(CreatePayload {
            circuit_id,
            onion_skin,
        });
        info!("Adding a new circuit with ID: {}", circuit_id);
        let mut circuits_lock = self.circuits.lock().await;
        circuits_lock.insert(circuit_id, relay_address);
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(serde_json::to_string(&create_payload)?.as_bytes())
            .await?;
        info!("Sent CREATE payload to relay at address: {}", relay_address);
        Ok(())
    }

    pub async fn send_extend_to_relay(
        &self,
        relay_address: SocketAddr,
        relay_address_2: SocketAddr,
    ) -> Result<()> {
        info!(
            "Extending circuit from relay at address: {} to relay at address: {}",
            relay_address, relay_address_2
        );
        let fetched_relays_lock = self.fetched_relays.lock().await;
        let relay = fetched_relays_lock
            .iter()
            .find(|r| r.address == relay_address_2)
            .unwrap();
        let rsa_public = Rsa::public_key_from_pem(&relay.rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = self.keys.dh.public_key().to_vec();
        let aes = generate_random_aes_key();
        let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
        let circuits_lock = self.circuits.lock().await;
        let circuit_id = circuits_lock
            .iter()
            .find(|(_, v)| **v == relay_address)
            .unwrap()
            .0;
        let extend_payload = Payload::Extend(ExtendPayload {
            circuit_id: *circuit_id,
            onion_skin,
            address: relay_address_2,
        });
        let handshakes_lock = self.handshakes.lock().await;
        let handshake = handshakes_lock.get(&relay_address).unwrap();
        let extend_payload_serialized = serde_json::to_string(&extend_payload)?;
        let extend_payload_serialized_encrypted = encrypt(
            Cipher::aes_256_ctr(),
            &handshake[0..32],
            None,
            extend_payload_serialized.as_bytes(),
        )?;
        info!(
            "Sending EXTEND payload to relay at address: {}",
            relay_address
        );
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&extend_payload_serialized_encrypted)
            .await?;
        info!("Sent Extend payload to relay at address: {}", relay_address);
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
