use crate::payloads::{CreatePayload, ExtendPayload, OnionSkin};
use crate::relay::RelayDescriptor;
use crate::relay_cell::RelayCell;
use crate::utils::{generate_random_aes_key, get_handshake_from_onion_skin, Connections};
use crate::{
    decrypt_buffer_with_aes, deserialize_payload_with_aes_keys, directory_address,
    encrypt_buffer_with_aes, serialize_payload_with_aes_keys, EstablishIntroductionPayload,
    EstablishRendezvousPayload, Event, Introduce1Payload, Payload, PayloadType,
};
use anyhow::Result;
use log::{error, info};
use openssl::bn::BigNum;
use openssl::dh::Dh;
use openssl::pkey::Private;
use openssl::rsa::Rsa;
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
    pub id: Uuid,
    pub nickname: String,
    pub rsa_public: Vec<u8>,
    pub introduction_points: Vec<(Uuid, SocketAddr)>,
}

pub struct UserKeys {
    pub rsa_private: Rsa<Private>,
    pub dh: Dh<Private>,
}

pub struct ConnectedUser {
    pub circuit_id: Uuid,
    pub rendezvous_cookie: Uuid,
    pub user_handshake: Vec<u8>,
}

pub struct User {
    pub user_descriptor: UserDescriptor,
    pub logs: Arc<Mutex<Vec<String>>>,
    fetched_relays: Mutex<Vec<RelayDescriptor>>,
    connections: Connections,
    keys: Arc<UserKeys>,
    handshakes: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>>,
    pub events_receiver: Arc<Mutex<Receiver<Event>>>,
    events_sender: Arc<Mutex<Sender<Event>>>,
    circuits: Arc<Mutex<HashMap<Uuid, Vec<SocketAddr>>>>,
    connected_users: Arc<Mutex<Vec<ConnectedUser>>>,
}

impl User {
    pub fn new(nickname: String) -> Self {
        let logs = Arc::new(Mutex::new(Vec::new()));
        info!("Creating new user: {:?}", nickname);
        let (events_sender, events_receiver) = tokio::sync::mpsc::channel(100);
        let rsa = Rsa::generate(2048).unwrap();
        let id = Uuid::new_v4();
        info!("User ID: {:?}", id);
        Self {
            user_descriptor: UserDescriptor {
                nickname,
                id,
                rsa_public: rsa.public_key_to_pem().unwrap(),
                introduction_points: Vec::new(),
            },
            logs,
            connections: Arc::new(Mutex::new(HashMap::new())),
            fetched_relays: Mutex::new(Vec::new()),
            keys: Arc::new(UserKeys {
                rsa_private: rsa.clone(),
                dh: Dh::get_2048_256().unwrap().generate_key().unwrap(),
            }),
            handshakes: Arc::new(Mutex::new(HashMap::new())),
            events_receiver: Arc::new(Mutex::new(events_receiver)),
            events_sender: Arc::new(Mutex::new(events_sender)),
            circuits: Arc::new(Mutex::new(HashMap::new())),
            connected_users: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_logs(&self) -> Vec<String> {
        self.logs.lock().await.clone()
    }

    pub fn add_introduction_point(&mut self, introduction_id: Uuid, address: SocketAddr) {
        self.user_descriptor
            .introduction_points
            .push((introduction_id, address));
    }

    pub fn get_user_descriptor(&self) -> UserDescriptor {
        self.user_descriptor.clone()
    }

    pub async fn start(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/users",
            directory_address().ip(),
            directory_address().port()
        );
        info!(
            "{} Registering user with directory server at URL: {}",
            self.user_descriptor.nickname, url
        );
        self.logs.lock().await.push(format!(
            "Registering user with directory server at URL: {}",
            url
        ));
        match client.post(&url).json(&self.user_descriptor).send().await {
            Ok(response) => {
                response.error_for_status_ref()?;
                info!(
                    "{} registered successfully with status: {}",
                    self.user_descriptor.nickname,
                    response.status()
                );
                self.logs.lock().await.push(format!(
                    "Registered successfully with status: {}",
                    response.status()
                ));
            }
            Err(e) => {
                error!(
                    "Failed to register user {}: {}",
                    self.user_descriptor.nickname, e
                );
                self.logs
                    .lock()
                    .await
                    .push(format!("Failed to register user: {}", e));
                return Err(e.into());
            }
        }
        Ok(())
    }

    /// Fetch all relays from the directory server
    pub async fn fetch_relays(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/relays",
            directory_address().ip(),
            directory_address().port()
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
        self.logs
            .lock()
            .await
            .push(format!("Connecting to relay {}", relay_discriptor.nickname));
        let stream = TcpStream::connect(relay_discriptor.address).await?;
        info!(
            "{} connected to relay {}",
            self.user_descriptor.nickname, relay_discriptor.nickname
        );
        self.logs
            .lock()
            .await
            .push(format!("Connected to relay {}", relay_discriptor.nickname));
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
            self.connected_users.clone(),
            self.user_descriptor.nickname.clone(),
            relay_discriptor.nickname.clone(),
            self.logs.clone(),
        );
        Ok(())
    }

    pub fn handle_read(
        mut read: OwnedReadHalf,
        keys: Arc<UserKeys>,
        events_sender: Arc<Mutex<Sender<Event>>>,
        circuits: Arc<Mutex<HashMap<Uuid, Vec<SocketAddr>>>>,
        handshakes: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>>,
        connected_users: Arc<Mutex<Vec<ConnectedUser>>>,
        nickname: String,
        relay_nickname: String,
        logs: Arc<Mutex<Vec<String>>>,
    ) {
        let relay_address = read.peer_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let keys = keys.clone();
                let mut buffer = [0; 50240];
                match read.read(&mut buffer).await {
                    Ok(0) => {}
                    Ok(n) => {
                        let relay_cell: RelayCell = serde_json::from_slice(&buffer[..n]).unwrap();
                        info!(
                            "{} received a relay cell from {:?} for circuit id {}",
                            nickname, relay_nickname, relay_cell.circuit_id
                        );
                        logs.lock().await.push(format!(
                            "Received a relay cell from {:?} for circuit id {}",
                            relay_nickname, relay_cell.circuit_id
                        ));
                        let mut circuits_lock = circuits.lock().await;
                        let mut handshakes_lock = handshakes.lock().await;
                        let mut connected_users = connected_users.lock().await;
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
                        logs.lock()
                            .await
                            .push(format!("Received payload: {:?}", payload.get_type()));
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
                                logs.lock().await.push(format!(
                                    "Handshake Successful: {}",
                                    hex::encode(&handshake[0..32])
                                ));
                                handshakes_lock.insert(relay_address, handshake);
                                circuits_lock.insert(relay_cell.circuit_id, vec![relay_address]);
                                info!(
                                    "{} added a new circuit with ID: {}",
                                    nickname, relay_cell.circuit_id
                                );
                                logs.lock().await.push(format!(
                                    "Added a new circuit with ID: {}",
                                    relay_cell.circuit_id
                                ));
                            }
                            Payload::Extended(extended_payload) => {
                                let handshake = keys
                                    .dh
                                    .compute_key(
                                        &BigNum::from_slice(&extended_payload.dh_key).unwrap(),
                                    )
                                    .unwrap();
                                info!("Handshake Successful: {}", hex::encode(&handshake[0..32]));
                                logs.lock().await.push(format!(
                                    "Handshake Successful: {}",
                                    hex::encode(&handshake[0..32])
                                ));
                                handshakes_lock.insert(extended_payload.address, handshake);
                                circuits_lock
                                    .get_mut(&relay_cell.circuit_id)
                                    .unwrap()
                                    .push(extended_payload.address);
                                info!(
                                    "{} extended circuit with ID: {} to relay at address: {}",
                                    nickname, relay_cell.circuit_id, extended_payload.address
                                );
                                logs.lock().await.push(format!(
                                    "Extended circuit with ID: {} to relay at address: {}",
                                    relay_cell.circuit_id, extended_payload.address
                                ));
                            }
                            Payload::Introduce2(introduce2_payload) => {
                                let handshake = get_handshake_from_onion_skin(
                                    introduce2_payload.onion_skin,
                                    &keys.dh,
                                    &keys.rsa_private,
                                );
                                let circuit_id = Uuid::new_v4();
                                info!(
                                    "Circuit id {} is used for the circuit to the rendezvous point {}",
                                    circuit_id, introduce2_payload.rendezvous_point_descriptor.nickname
                                );
                                logs.lock().await.push(format!(
                                    "Circuit id {} is used for the circuit to the rendezvous point {}",
                                    circuit_id, introduce2_payload.rendezvous_point_descriptor.nickname
                                ));
                                let connected_user = ConnectedUser {
                                    rendezvous_cookie: introduce2_payload.rendezvous_cookie,
                                    circuit_id,
                                    user_handshake: handshake,
                                };
                                connected_users.push(connected_user);
                            }
                            Payload::Rendezvous2(rendezvous2_payload) => {
                                let handshake = keys
                                    .dh
                                    .compute_key(
                                        &BigNum::from_slice(&rendezvous2_payload.dh_key).unwrap(),
                                    )
                                    .unwrap();
                                info!("Handshake Successful: {}", hex::encode(&handshake[0..32]));
                                logs.lock().await.push(format!(
                                    "Handshake Successful: {}",
                                    hex::encode(&handshake[0..32])
                                ));
                                let connected_user = ConnectedUser {
                                    rendezvous_cookie: rendezvous2_payload.rendezvous_cookie,
                                    circuit_id: relay_cell.circuit_id,
                                    user_handshake: handshake,
                                };
                                connected_users.push(connected_user);
                            }
                            Payload::Data(data_payload) => {
                                info!(
                                    "{} received data from relay at address: {}",
                                    nickname, relay_address
                                );
                                logs.lock().await.push(format!(
                                    "Received data from relay at address: {}",
                                    relay_address
                                ));
                                let connected_user = connected_users
                                    .iter()
                                    .find(|u| u.circuit_id == relay_cell.circuit_id)
                                    .unwrap();
                                let decrypted_data = decrypt_buffer_with_aes(
                                    &connected_user.user_handshake,
                                    &data_payload.data,
                                )
                                .unwrap();
                                info!(
                                    "Received String from user with rendezvous cookie {}: {:?}",
                                    connected_user.rendezvous_cookie,
                                    String::from_utf8(decrypted_data.clone()).unwrap()
                                );
                                logs.lock().await.push(format!(
                                    "Received String from user with rendezvous cookie {}: {:?}",
                                    connected_user.rendezvous_cookie,
                                    String::from_utf8(decrypted_data).unwrap()
                                ));
                            }
                            Payload::EstablishedIntroduction(_) => {
                                info!("{} established an introduction point", nickname);
                                logs.lock()
                                    .await
                                    .push(format!("Established an introduction point"));
                            }
                            Payload::EstablishedRendezvous(_) => {
                                info!("{} established a rendezvous point", nickname);
                                logs.lock()
                                    .await
                                    .push(format!("Established a rendezvous point"));
                            }
                            Payload::Connected(_) => {
                                info!("{} connected to a relay", nickname);
                                logs.lock().await.push(format!("Connected to a relay"));
                            }
                            Payload::IntroduceAck(_) => {
                                info!("{} received an introduction ack", nickname);
                                logs.lock()
                                    .await
                                    .push(format!("Received an introduction ack"));
                            }
                            _ => {
                                error!("{} received an unexpected payload", nickname);
                                logs.lock()
                                    .await
                                    .push(format!("Received an unexpected payload"));
                            }
                        }
                        let _ = events_sender
                            .lock()
                            .await
                            .send(Event(payload_type, relay_address))
                            .await;
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
        self.logs.lock().await.push(format!(
            "Sending CREATE payload to: {}",
            relay_descriptor.nickname
        ));
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
        if let None = connections_lock.get_mut(&relay_address) {
            info!(
                "{} connecting to relay {}",
                self.user_descriptor.nickname, relay_descriptor.nickname
            );
            self.logs
                .lock()
                .await
                .push(format!("Connecting to relay {}", relay_descriptor.nickname));
            let stream = TcpStream::connect(relay_descriptor.address).await?;
            info!(
                "{} connected to relay {}",
                self.user_descriptor.nickname, relay_descriptor.nickname
            );
            self.logs
                .lock()
                .await
                .push(format!("Connected to relay {}", relay_descriptor.nickname));
            let (read, write) = stream.into_split();
            let write = Arc::new(Mutex::new(write));
            connections_lock.insert(relay_address, write);
            Self::handle_read(
                read,
                self.keys.clone(),
                self.events_sender.clone(),
                self.circuits.clone(),
                self.handshakes.clone(),
                self.connected_users.clone(),
                self.user_descriptor.nickname.clone(),
                relay_descriptor.nickname.clone(),
                self.logs.clone(),
            );
        };
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
        self.logs.lock().await.push(format!(
            "Sent CREATE payload to: {}",
            relay_descriptor.nickname
        ));
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
        self.logs.lock().await.push(format!(
            "Extending circuit from relay at address: {} to relay at address: {}",
            relay_address, relay_address_2
        ));
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
        self.logs.lock().await.push(format!(
            "Sending EXTEND payload to relay {}",
            relay_descriptor.nickname
        ));
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell).unwrap())
            .await?;
        info!("Sent EXTEND payload to relay {}", relay_descriptor.nickname);
        self.logs.lock().await.push(format!(
            "Sent EXTEND payload to relay {}",
            relay_descriptor.nickname
        ));
        Ok(())
    }

    pub async fn send_establish_rendezvous_to_relay(
        &self,
        relay_address: SocketAddr,
        rendezvous_cookie: Uuid,
        circuit_id: Uuid,
    ) -> Result<()> {
        info!(
            "Sending ESTABLISH_INTRO to relay at address: {}",
            relay_address
        );
        self.logs.lock().await.push(format!(
            "Sending ESTABLISH_INTRO to relay at address: {}",
            relay_address
        ));
        let establish_intro_payload =
            Payload::EstablishRendezvous(EstablishRendezvousPayload { rendezvous_cookie });
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
            serialize_payload_with_aes_keys(handshakes, &establish_intro_payload).unwrap();
        let relay_cell = RelayCell {
            circuit_id,
            payload: encrypted_payload,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        info!(
            "Sent ESTABLISH_INTRO payload to relay at address: {}",
            relay_address
        );
        self.logs.lock().await.push(format!(
            "Sent ESTABLISH_INTRO payload to relay at address: {}",
            relay_address
        ));
        Ok(())
    }

    pub async fn send_establish_introduction_to_relay(
        &self,
        relay_address: SocketAddr,
        introduction_id: Uuid,
        circuit_id: Uuid,
    ) -> Result<()> {
        info!(
            "{} Sending ESTABLISH_INTRO to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sending ESTABLISH_INTRO to relay at address: {}",
            relay_address
        ));
        let establish_intro_payload =
            Payload::EstablishIntroduction(EstablishIntroductionPayload {
                introduction_id,
                rsa_publickey: self.user_descriptor.rsa_public.clone(),
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
            serialize_payload_with_aes_keys(handshakes, &establish_intro_payload).unwrap();
        let relay_cell = RelayCell {
            circuit_id,
            payload: encrypted_payload,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        info!(
            "{} Sent ESTABLISH_INTRO payload to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sent ESTABLISH_INTRO payload to relay at address: {}",
            relay_address
        ));
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

    pub async fn send_begin_to_relay(
        &self,
        relay_address: SocketAddr,
        circuit_id: Uuid,
        stream_id: Uuid,
        begin_relay_address: SocketAddr,
    ) -> Result<()> {
        info!(
            "{} Sending BEGIN to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sending BEGIN to relay at address: {}",
            relay_address
        ));
        let relay_descriptor = self
            .fetched_relays
            .lock()
            .await
            .iter()
            .find(|r| r.address == begin_relay_address)
            .unwrap()
            .clone();
        let begin_payload = Payload::Begin(crate::BeginPayload {
            stream_id,
            relay_descriptor,
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
            serialize_payload_with_aes_keys(handshakes, &begin_payload).unwrap();
        let relay_cell = RelayCell {
            circuit_id,
            payload: encrypted_payload,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        info!(
            "{} Sent BEGIN payload to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sent BEGIN payload to relay at address: {}",
            relay_address
        ));
        Ok(())
    }

    pub async fn send_introduce1_to_relay(
        &self,
        relay_address: SocketAddr,
        introduction_id: Uuid,
        stream_id: Uuid,
        rendezvous_point_descriptor: SocketAddr,
        rendezvous_cookie: Uuid,
        introduction_rsa_public: Vec<u8>,
        circuit_id: Uuid,
    ) -> Result<()> {
        info!(
            "{} Sending INTRODUCE1 to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sending INTRODUCE1 to relay at address: {}",
            relay_address
        ));
        let relay_descriptor = self
            .fetched_relays
            .lock()
            .await
            .iter()
            .find(|r| r.address == rendezvous_point_descriptor)
            .unwrap()
            .clone();
        let rsa_public = Rsa::public_key_from_pem(&introduction_rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = self.keys.dh.public_key().to_vec();
        let aes = generate_random_aes_key();
        let onion_skin = OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap());
        let introduce1_payload = Introduce1Payload {
            stream_id,
            introduction_id,
            rendezvous_point_descriptor: relay_descriptor,
            rendezvous_cookie,
            onion_skin,
        };
        let introduce1_payload = Payload::Introduce1(introduce1_payload);
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
            serialize_payload_with_aes_keys(handshakes, &introduce1_payload).unwrap();
        let relay_cell = RelayCell {
            circuit_id,
            payload: encrypted_payload,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        info!(
            "{} Sent INTRODUCE1 payload to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sent INTRODUCE1 payload to relay at address: {}",
            relay_address
        ));
        Ok(())
    }

    pub async fn update_introduction_points(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/users/{}/introduction_points",
            directory_address().ip(),
            directory_address().port(),
            self.user_descriptor.id
        );
        info!(
            "{} updating introduction points",
            self.user_descriptor.nickname
        );
        self.logs
            .lock()
            .await
            .push(format!("Updating introduction points"));
        let introduction_points = self.user_descriptor.introduction_points.clone();
        match client.post(&url).json(&introduction_points).send().await {
            Ok(response) => {
                response.error_for_status_ref()?;
                info!(
                    "{} updated introduction points successfully",
                    self.user_descriptor.nickname,
                );
                self.logs
                    .lock()
                    .await
                    .push(format!("Updated introduction points successfully"));
            }
            Err(e) => {
                error!(
                    "Failed to update introduction points for user {}: {}",
                    self.user_descriptor.nickname, e
                );
                self.logs.lock().await.push(format!(
                    "Failed to update introduction points for user: {}",
                    e
                ));
                return Err(e.into());
            }
        }
        Ok(())
    }

    pub async fn establish_circuit(
        &self,
        circuit_id: Uuid,
        relay_address_1: SocketAddr,
        relay_address_2: SocketAddr,
        relay_address_3: SocketAddr,
    ) -> Result<()> {
        let fetched_relays_lock = self.fetched_relays.lock().await;
        let relay_descriptor = fetched_relays_lock
            .iter()
            .find(|r| r.address == relay_address_1)
            .unwrap()
            .clone();
        drop(fetched_relays_lock);
        self.connect_to_relay(relay_descriptor).await?;
        self.send_create_to_relay(relay_address_1, circuit_id)
            .await
            .unwrap();
        self.listen_for_event(Event(PayloadType::Created, relay_address_1))
            .await
            .unwrap();
        self.send_extend_to_relay(relay_address_1, relay_address_2, circuit_id)
            .await
            .unwrap();
        self.listen_for_event(Event(PayloadType::Extended, relay_address_1))
            .await
            .unwrap();
        self.send_extend_to_relay(relay_address_1, relay_address_3, circuit_id)
            .await
            .unwrap();
        self.listen_for_event(Event(PayloadType::Extended, relay_address_1))
            .await
            .unwrap();
        info!(
            "{} established a circuit with 3 relays, {} {} {}",
            self.user_descriptor.nickname, relay_address_1, relay_address_2, relay_address_3
        );
        self.logs.lock().await.push(format!(
            "Established a circuit with 3 relays, {} {} {}",
            relay_address_1, relay_address_2, relay_address_3
        ));
        Ok(())
    }

    pub async fn send_rendezvous1_to_relay(
        &self,
        relay_address: SocketAddr,
        rendezvous_cookie: Uuid,
        circuit_id: Uuid,
    ) -> Result<()> {
        info!(
            "{} Sending RENDEZVOUS1 to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sending RENDEZVOUS1 to relay at address: {}",
            relay_address
        ));
        let rendezvous1_payload = Payload::Rendezvous1(crate::Rendezvous1Payload {
            rendezvous_cookie,
            dh_key: self.keys.dh.public_key().to_vec(),
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
            serialize_payload_with_aes_keys(handshakes, &rendezvous1_payload).unwrap();
        let relay_cell = RelayCell {
            circuit_id,
            payload: encrypted_payload,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        info!(
            "{} Sent RENDEZVOUS1 payload to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sent RENDEZVOUS1 payload to relay at address: {}",
            relay_address
        ));
        Ok(())
    }

    pub async fn send_data_to_relay(
        &self,
        relay_address: SocketAddr,
        rendezvous_cookie: Uuid,
        circuit_id: Uuid,
        data: Vec<u8>,
    ) -> Result<()> {
        info!(
            "{} Sending DATA to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sending DATA to relay at address: {}",
            relay_address
        ));
        let user_handshake = self
            .connected_users
            .lock()
            .await
            .iter()
            .find(|u| u.rendezvous_cookie == rendezvous_cookie)
            .unwrap()
            .user_handshake
            .clone();
        let encrypted_data = encrypt_buffer_with_aes(&user_handshake, &data).unwrap();
        let data_payload: Payload = Payload::Data(crate::DataPayload {
            data: encrypted_data,
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
        let encrypted_payload = serialize_payload_with_aes_keys(handshakes, &data_payload).unwrap();
        let relay_cell = RelayCell {
            circuit_id,
            payload: encrypted_payload,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        info!(
            "{} Sent DATA payload to relay at address: {}",
            self.user_descriptor.nickname, relay_address
        );
        self.logs.lock().await.push(format!(
            "Sent DATA payload to relay at address: {}",
            relay_address
        ));
        Ok(())
    }

    pub async fn get_circuit_id_for_rendezvous(&self, rendezvous_cookie: Uuid) -> Uuid {
        self.connected_users
            .lock()
            .await
            .iter()
            .find(|u| u.rendezvous_cookie == rendezvous_cookie)
            .unwrap()
            .circuit_id
    }
}
