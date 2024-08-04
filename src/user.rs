use crate::payloads::{CreatePayload, ExtendPayload};
use crate::relay::RelayDescriptor;
use crate::relay_cell::RelayCell;
use crate::{
    decrypt_buffer_with_aes, encrypt_buffer_with_aes, generate_random_aes_key,
    get_handshake_from_onion_skin, Connections, Directory, EstablishIntroductionPayload,
    EstablishRendezvousPayload, Event, Introduce1Payload, Keys, Logger, OnionSkin, Payload,
    PayloadType,
};
use anyhow::Result;
use openssl::bn::BigNum;
use openssl::dh::Dh;
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
pub struct ConnectedUser {
    pub rendezvous_cookie: Uuid,
    pub user_handshake: Vec<u8>,
}

pub struct User {
    pub user_descriptor: UserDescriptor,
    fetched_relays: Mutex<Vec<RelayDescriptor>>,
    connections: Connections,
    keys: Arc<Keys>,
    handshakes: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>>,
    pub events_receiver: Arc<Mutex<Receiver<Event>>>,
    events_sender: Arc<Mutex<Sender<Event>>>,
    circuits: Arc<Mutex<HashMap<Uuid, Vec<SocketAddr>>>>,
    connected_users: Arc<Mutex<Vec<ConnectedUser>>>,
}

impl User {
    pub fn new(nickname: String) -> Self {
        Logger::info(&nickname, "Creating new user");
        let (events_sender, events_receiver) = tokio::sync::mpsc::channel(100);
        let rsa = Rsa::generate(2048).unwrap();
        let id = Uuid::new_v4();
        Logger::info(&nickname, format!("User ID: {:?}", id));
        Self {
            user_descriptor: UserDescriptor {
                nickname,
                id,
                rsa_public: rsa.public_key_to_pem().unwrap(),
                introduction_points: Vec::new(),
            },
            connections: Arc::new(Mutex::new(HashMap::new())),
            fetched_relays: Mutex::new(Vec::new()),
            keys: Arc::new(Keys {
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

    pub fn add_introduction_point(&mut self, introduction_id: Uuid, address: SocketAddr) {
        self.user_descriptor
            .introduction_points
            .push((introduction_id, address));
    }

    #[cfg(test)]
    pub fn get_user_descriptor(&self) -> UserDescriptor {
        self.user_descriptor.clone()
    }

    pub async fn start(&self) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            "Registering user with directory server",
        );
        Directory::publish_user(self.user_descriptor.clone());
        Logger::info(&self.user_descriptor.nickname, "Registered successfully");
        Ok(())
    }

    /// Fetch all relays from the directory server
    pub async fn fetch_relays(&self) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            "Fetching relays from directory",
        );
        let relays_fetched: Vec<RelayDescriptor> = Directory::get_relays();
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Fetched {:?} relays", relays_fetched.len()),
        );
        let mut relays = self.fetched_relays.lock().await;
        *relays = relays_fetched;
        Ok(())
    }

    /// Connect to a relay server
    pub async fn connect_to_relay(&self, relay_discriptor: RelayDescriptor) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Connecting to relay {}", relay_discriptor.nickname),
        );
        let stream = TcpStream::connect(relay_discriptor.address).await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Connected to relay {}", relay_discriptor.nickname),
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
            self.connected_users.clone(),
            self.user_descriptor.nickname.clone(),
            relay_discriptor.nickname.clone(),
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn handle_read(
        mut read: OwnedReadHalf,
        keys: Arc<Keys>,
        events_sender: Arc<Mutex<Sender<Event>>>,
        circuits: Arc<Mutex<HashMap<Uuid, Vec<SocketAddr>>>>,
        handshakes: Arc<Mutex<HashMap<SocketAddr, Vec<u8>>>>,
        connected_users: Arc<Mutex<Vec<ConnectedUser>>>,
        nickname: String,
        relay_nickname: String,
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
                        Logger::info(
                            &nickname,
                            format!(
                                "Received a relay cell from {:?} for circuit id {}",
                                relay_nickname, relay_cell.circuit_id
                            ),
                        );
                        let mut circuits_lock = circuits.lock().await;
                        let mut handshakes_lock = handshakes.lock().await;
                        let mut connected_users = connected_users.lock().await;
                        let payload: Payload = if let Some(circuit) =
                            circuits_lock.get(&relay_cell.circuit_id)
                        {
                            let mut vec_handshakes = vec![];
                            for relay in circuit {
                                vec_handshakes.push(handshakes_lock.get(relay).unwrap().clone());
                            }
                            let mut buffer = relay_cell.payload.clone();
                            for handshake in vec_handshakes.iter() {
                                Logger::info(
                                    &nickname,
                                    format!(
                                        "Decoding with handshake: {}",
                                        hex::encode(&handshake[0..32])
                                    ),
                                );
                                buffer = decrypt_buffer_with_aes(handshake, &buffer).unwrap();
                            }
                            serde_json::from_slice(&buffer).unwrap()
                        } else {
                            serde_json::from_slice(&relay_cell.payload).unwrap()
                        };
                        Logger::info(
                            &nickname,
                            &format!("Received payload: {:?}", payload.get_type()),
                        );
                        let payload_type = payload.get_type();
                        match payload {
                            Payload::Created(created_payload) => {
                                let handshake = keys
                                    .dh
                                    .compute_key(
                                        &BigNum::from_slice(&created_payload.dh_key).unwrap(),
                                    )
                                    .unwrap();
                                Logger::info(
                                    &nickname,
                                    &format!(
                                        "Handshake Successful: {}",
                                        hex::encode(&handshake[0..32])
                                    ),
                                );
                                handshakes_lock.insert(relay_address, handshake);
                                circuits_lock.insert(relay_cell.circuit_id, vec![relay_address]);
                                Logger::info(
                                    &nickname,
                                    format!(
                                        "Added a new circuit with ID: {}",
                                        relay_cell.circuit_id
                                    ),
                                );
                            }
                            Payload::Extended(extended_payload) => {
                                let handshake = keys
                                    .dh
                                    .compute_key(
                                        &BigNum::from_slice(&extended_payload.dh_key).unwrap(),
                                    )
                                    .unwrap();
                                Logger::info(
                                    &nickname,
                                    &format!(
                                        "Handshake Successful: {}",
                                        hex::encode(&handshake[0..32])
                                    ),
                                );
                                handshakes_lock.insert(extended_payload.address, handshake);
                                circuits_lock
                                    .get_mut(&relay_cell.circuit_id)
                                    .unwrap()
                                    .push(extended_payload.address);
                                Logger::info(
                                    &nickname,
                                    format!(
                                        "Extended circuit with ID: {} to relay at address: {}",
                                        relay_cell.circuit_id, extended_payload.address
                                    ),
                                );
                            }
                            Payload::Introduce2(introduce2_payload) => {
                                let handshake = get_handshake_from_onion_skin(
                                    introduce2_payload.onion_skin,
                                    &keys.dh,
                                    &keys.rsa_private,
                                )
                                .unwrap();
                                Logger::info(&nickname,format!(
                                    "Circuit id {} is used for the circuit to the rendezvous point {}",
                                    relay_cell.circuit_id, introduce2_payload.rendezvous_point_descriptor.nickname
                                ));
                                let connected_user = ConnectedUser {
                                    rendezvous_cookie: introduce2_payload.rendezvous_cookie,
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
                                Logger::info(
                                    &nickname,
                                    &format!(
                                        "Handshake Successful: {}",
                                        hex::encode(&handshake[0..32])
                                    ),
                                );
                                let connected_user = ConnectedUser {
                                    rendezvous_cookie: rendezvous2_payload.rendezvous_cookie,
                                    user_handshake: handshake,
                                };
                                connected_users.push(connected_user);
                            }
                            Payload::Data(data_payload) => {
                                Logger::info(
                                    &nickname,
                                    format!(
                                        "Received data from relay at address: {}",
                                        relay_address
                                    ),
                                );
                                let connected_user = connected_users
                                    .iter()
                                    .find(|u| u.rendezvous_cookie == data_payload.rendezvous_cookie)
                                    .unwrap();
                                let decrypted_data = decrypt_buffer_with_aes(
                                    &connected_user.user_handshake,
                                    &data_payload.data,
                                )
                                .unwrap();
                                Logger::info(
                                    &nickname,
                                    format!(
                                        "Received String from user with rendezvous cookie {}: {:?}",
                                        connected_user.rendezvous_cookie,
                                        String::from_utf8(decrypted_data.clone()).unwrap()
                                    ),
                                );
                            }
                            Payload::EstablishedIntroduction(_) => {
                                Logger::info(&nickname, "Established an introduction point");
                            }
                            Payload::EstablishedRendezvous(_) => {
                                Logger::info(&nickname, "Established a rendezvous point");
                            }
                            Payload::Connected(_) => {
                                Logger::info(&nickname, "Connected to a relay");
                            }
                            Payload::IntroduceAck(_) => {
                                Logger::info(
                                    &nickname,
                                    "Received an IntroduceAck payload from a relay",
                                );
                            }
                            _ => {
                                Logger::error(&nickname, "Received an unknown payload");
                            }
                        }
                        let _ = events_sender
                            .lock()
                            .await
                            .send(Event(payload_type, relay_address))
                            .await;
                    }
                    Err(e) => {
                        Logger::error(&nickname, format!("Failed to read from relay: {}", e));
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
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sending CREATE payload to: {}", relay_descriptor.nickname,),
        );

        let rsa_public = Rsa::public_key_from_pem(&relay_descriptor.rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = self.keys.dh.public_key().to_vec();
        let aes = generate_random_aes_key();
        let onion_skin =
            OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap()).unwrap();
        let create_payload = Payload::Create(CreatePayload { onion_skin });
        let relay_cell = RelayCell {
            circuit_id,
            payload: serde_json::to_vec(&create_payload)?,
        };
        let mut connections_lock = self.connections.lock().await;
        if connections_lock.get_mut(&relay_address).is_none() {
            Logger::info(
                &self.user_descriptor.nickname,
                format!("Connecting to relay {}", relay_descriptor.nickname),
            );
            let stream = TcpStream::connect(relay_descriptor.address).await?;
            Logger::info(
                &self.user_descriptor.nickname,
                format!("connected to relay {}", relay_descriptor.nickname),
            );
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
            );
        };
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sent CREATE payload to: {}", relay_descriptor.nickname),
        );
        Ok(())
    }

    pub async fn send_extend_to_relay(
        &self,
        relay_address: SocketAddr,
        relay_address_2: SocketAddr,
        circuit_id: Uuid,
    ) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Extending circuit from relay at address: {} to relay at address: {}",
                relay_address, relay_address_2
            ),
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
        let onion_skin =
            OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap()).unwrap();
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
        let mut buffer: Vec<u8> = serde_json::to_vec(&extend_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.user_descriptor.nickname,
                format!(
                    "Encoding with handshake: {}",
                    hex::encode(&handshake[0..32])
                ),
            );
            buffer = encrypt_buffer_with_aes(handshake, &buffer).unwrap();
        }
        let relay_cell = RelayCell {
            circuit_id,
            payload: buffer,
        };
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Sending EXTEND payload to relay {}",
                relay_descriptor.nickname
            ),
        );
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell).unwrap())
            .await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sent EXTEND payload to relay {}", relay_descriptor.nickname),
        );
        Ok(())
    }

    pub async fn send_establish_rendezvous_to_relay(
        &self,
        relay_address: SocketAddr,
        rendezvous_cookie: Uuid,
        circuit_id: Uuid,
    ) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Sending ESTABLISH_INTRO to relay at address: {}",
                relay_address
            ),
        );
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
        let mut buffer = serde_json::to_vec(&establish_intro_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.user_descriptor.nickname,
                format!(
                    "Encoding with handshake: {}",
                    hex::encode(&handshake[0..32])
                ),
            );
            buffer = encrypt_buffer_with_aes(handshake, &buffer).unwrap();
        }
        let relay_cell = RelayCell {
            circuit_id,
            payload: buffer,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Sent ESTABLISH_INTRO payload to relay at address: {}",
                relay_address
            ),
        );
        Ok(())
    }

    pub async fn send_establish_introduction_to_relay(
        &self,
        relay_address: SocketAddr,
        introduction_id: Uuid,
        circuit_id: Uuid,
    ) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Sending ESTABLISH_INTRO to relay at address: {}",
                relay_address
            ),
        );
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
        let mut buffer = serde_json::to_vec(&establish_intro_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.user_descriptor.nickname,
                format!(
                    "Encoding with handshake: {}",
                    hex::encode(&handshake[0..32])
                ),
            );
            buffer = encrypt_buffer_with_aes(handshake, &buffer).unwrap();
        }
        let relay_cell = RelayCell {
            circuit_id,
            payload: buffer,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Sent ESTABLISH_INTRO payload to relay at address: {}",
                relay_address
            ),
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

    pub async fn send_begin_to_relay(
        &self,
        relay_address: SocketAddr,
        circuit_id: Uuid,
        stream_id: Uuid,
        begin_relay_address: SocketAddr,
    ) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sending BEGIN to relay at address: {}", relay_address),
        );
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
        let mut buffer = serde_json::to_vec(&begin_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.user_descriptor.nickname,
                format!(
                    "Encoding with handshake: {}",
                    hex::encode(&handshake[0..32])
                ),
            );
            buffer = encrypt_buffer_with_aes(handshake, &buffer).unwrap();
        }
        let relay_cell = RelayCell {
            circuit_id,
            payload: buffer,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sent BEGIN payload to relay at address: {}", relay_address),
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
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
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sending INTRODUCE1 to relay at address: {}", relay_address),
        );
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
        let onion_skin =
            OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap()).unwrap();
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
        let mut buffer = serde_json::to_vec(&introduce1_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.user_descriptor.nickname,
                format!(
                    "Encoding with handshake: {}",
                    hex::encode(&handshake[0..32])
                ),
            );
            buffer = encrypt_buffer_with_aes(handshake, &buffer).unwrap();
        }
        let relay_cell = RelayCell {
            circuit_id,
            payload: buffer,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Sent INTRODUCE1 payload to relay at address: {}",
                relay_address
            ),
        );
        Ok(())
    }

    pub async fn update_introduction_points(&self) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            "Updating introduction points",
        );
        let introduction_points = self.user_descriptor.introduction_points.clone();
        Directory::update_user_introduction_points(self.user_descriptor.id, introduction_points);
        Logger::info(
            &self.user_descriptor.nickname,
            "Updated introduction points successfully",
        );
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
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Established a circuit with 3 relays, {} {} {}",
                relay_address_1, relay_address_2, relay_address_3
            ),
        );
        Ok(())
    }

    pub async fn send_rendezvous1_to_relay(
        &self,
        relay_address: SocketAddr,
        rendezvous_cookie: Uuid,
        circuit_id: Uuid,
    ) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sending RENDEZVOUS1 to relay at address: {}", relay_address),
        );
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
        let mut buffer = serde_json::to_vec(&rendezvous1_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.user_descriptor.nickname,
                format!(
                    "Encoding with handshake: {}",
                    hex::encode(&handshake[0..32])
                ),
            );
            buffer = encrypt_buffer_with_aes(handshake, &buffer).unwrap();
        }
        let relay_cell = RelayCell {
            circuit_id,
            payload: buffer,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!(
                "Sent RENDEZVOUS1 payload to relay at address: {}",
                relay_address
            ),
        );
        Ok(())
    }

    pub async fn send_data_to_relay(
        &self,
        relay_address: SocketAddr,
        rendezvous_cookie: Uuid,
        circuit_id: Uuid,
        data: Vec<u8>,
    ) -> Result<()> {
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sending DATA to relay at address: {}", relay_address),
        );
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
            rendezvous_cookie,
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
        let mut buffer = serde_json::to_vec(&data_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.user_descriptor.nickname,
                format!(
                    "Encoding with handshake: {}",
                    hex::encode(&handshake[0..32])
                ),
            );
            buffer = encrypt_buffer_with_aes(handshake, &buffer).unwrap();
        }
        let relay_cell = RelayCell {
            circuit_id,
            payload: buffer,
        };
        let mut connections_lock = self.connections.lock().await;
        let stream = connections_lock.get_mut(&relay_address).unwrap();
        stream
            .lock()
            .await
            .write_all(&serde_json::to_vec(&relay_cell)?)
            .await?;
        Logger::info(
            &self.user_descriptor.nickname,
            format!("Sent DATA payload to relay at address: {}", relay_address),
        );
        Ok(())
    }
}
