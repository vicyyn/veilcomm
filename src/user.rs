use crate::payloads::{CreatePayload, ExtendPayload};
use crate::relay_cell::RelayCell;
use crate::{
    decrypt_buffer_with_aes, encrypt_buffer_with_aes, generate_random_aes_key,
    get_handshake_from_onion_skin, CircuitId, Communication, Directory,
    EstablishIntroductionPayload, EstablishRendezvousPayload, Event, Handshake, Introduce1Payload,
    IntroductionPointId, Keys, Logger, OnionSkin, Payload, PayloadType, RelayId,
    RendezvousCookieId, StreamId, UserId, UserState,
};
use anyhow::Result;
use openssl::bn::BigNum;
use openssl::dh::Dh;
use openssl::rsa::Rsa;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserDescriptor {
    pub id: UserId,
    pub nickname: String,
    pub rsa_public: Vec<u8>,
    pub introduction_points: HashMap<IntroductionPointId, RelayId>,
}
pub struct ConnectedUser {
    pub rendezvous_cookie: RendezvousCookieId,
    pub user_handshake: Handshake,
}

pub struct InternalState {
    keys: Keys,
    handshakes: HashMap<RelayId, Handshake>,
    events_sender: Sender<Event>,
    circuits: HashMap<CircuitId, Vec<RelayId>>,
    connected_users: HashMap<RendezvousCookieId, Handshake>,
    stream_ids: HashMap<StreamId, RelayId>,
}

pub struct User {
    nickname: String,
    id: UserId,
    rsa_public: Vec<u8>,
    pub events_receiver: Receiver<Event>,
    pub user_descriptor: UserDescriptor,
    internal_state: Arc<Mutex<InternalState>>,
}

impl User {
    pub fn new(nickname: String) -> Self {
        Logger::info(&nickname, "Creating new user");
        let (events_sender, events_receiver) = mpsc::channel();
        let rsa = Rsa::generate(2048).unwrap();
        let id = UserId::new_v4();
        Logger::info(&nickname, format!("User ID: {:?}", id));
        Self {
            nickname: nickname.clone(),
            id,
            rsa_public: rsa.public_key_to_pem().unwrap(),
            events_receiver,
            user_descriptor: UserDescriptor {
                nickname,
                id,
                rsa_public: rsa.public_key_to_pem().unwrap(),
                introduction_points: HashMap::new(),
            },
            internal_state: Arc::new(Mutex::new(InternalState {
                keys: Keys {
                    rsa_private: rsa.clone(),
                    dh: Dh::get_2048_256().unwrap().generate_key().unwrap(),
                },
                handshakes: HashMap::new(),
                events_sender,
                circuits: HashMap::new(),
                connected_users: HashMap::new(),
                stream_ids: HashMap::new(),
            })),
        }
    }

    pub fn get_state(&self) -> UserState {
        let internal_state_lock = self.internal_state.lock().unwrap();
        UserState {
            id: self.id,
            nickname: self.nickname.clone(),
            introduction_points: self.user_descriptor.introduction_points.clone(),
            circuits: internal_state_lock.circuits.clone(),
            handshakes: internal_state_lock.handshakes.clone(),
            connected_users: internal_state_lock.connected_users.clone(),
            streams: internal_state_lock.stream_ids.clone().into_iter().collect(),
            logs: Logger::get_logs(self.nickname.clone()),
        }
    }

    pub fn start(&self) {
        let id = self.id;
        let nickname = self.nickname.clone();
        let rsa_public = self.rsa_public.clone();

        Logger::info(&nickname, "Registering user with directory server");
        Directory::publish_user(UserDescriptor {
            id,
            nickname: nickname.clone(),
            rsa_public,
            introduction_points: HashMap::new(),
        });
        Logger::info(&nickname, "Registered successfully");

        Logger::info(&nickname, "Registering with communication server");
        let receiver = Communication::register(id);
        Logger::info(
            &nickname,
            "Successfully registered with the communication server",
        );

        let internal_state = self.internal_state.clone();
        thread::spawn(move || loop {
            match receiver.recv() {
                Ok((sender_id, relay_cell)) => {
                    Logger::info(
                        &nickname,
                        format!(
                            "Received a relay cell from {:?} for circuit id {}",
                            sender_id, relay_cell.circuit_id
                        ),
                    );
                    let mut internal_state_lock = internal_state.lock().unwrap();
                    let payload: Payload = if let Some(circuit) =
                        internal_state_lock.circuits.get(&relay_cell.circuit_id)
                    {
                        let mut vec_handshakes = vec![];
                        for relay in circuit {
                            vec_handshakes
                                .push(internal_state_lock.handshakes.get(relay).unwrap().clone());
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
                            let handshake = internal_state_lock
                                .keys
                                .dh
                                .compute_key(&BigNum::from_slice(&created_payload.dh_key).unwrap())
                                .unwrap();
                            Logger::info(
                                &nickname,
                                &format!(
                                    "Handshake Successful: {}",
                                    hex::encode(&handshake[0..32])
                                ),
                            );
                            internal_state_lock.handshakes.insert(sender_id, handshake);
                            internal_state_lock
                                .circuits
                                .insert(relay_cell.circuit_id, vec![sender_id]);
                            Logger::info(
                                &nickname,
                                format!("Added a new circuit with ID {}", relay_cell.circuit_id),
                            );
                        }
                        Payload::Extended(extended_payload) => {
                            let handshake = internal_state_lock
                                .keys
                                .dh
                                .compute_key(&BigNum::from_slice(&extended_payload.dh_key).unwrap())
                                .unwrap();
                            Logger::info(
                                &nickname,
                                &format!(
                                    "Handshake Successful: {}",
                                    hex::encode(&handshake[0..32])
                                ),
                            );
                            internal_state_lock
                                .handshakes
                                .insert(extended_payload.extend_to, handshake);
                            internal_state_lock
                                .circuits
                                .get_mut(&relay_cell.circuit_id)
                                .unwrap()
                                .push(extended_payload.extend_to);
                            Logger::info(
                                &nickname,
                                format!(
                                    "Extended circuit with ID {} to relay {}",
                                    relay_cell.circuit_id, extended_payload.extend_to
                                ),
                            );
                        }
                        Payload::Introduce2(introduce2_payload) => {
                            let handshake = get_handshake_from_onion_skin(
                                introduce2_payload.onion_skin,
                                &internal_state_lock.keys.dh,
                                &internal_state_lock.keys.rsa_private,
                            )
                            .unwrap();
                            Logger::info(&nickname,format!(
                                    "Circuit id {} is used for the circuit to the rendezvous point {}",
                                    relay_cell.circuit_id, introduce2_payload.rendezvous_point_descriptor.nickname
                                ));
                            internal_state_lock
                                .connected_users
                                .insert(introduce2_payload.rendezvous_cookie, handshake);
                        }
                        Payload::Rendezvous2(rendezvous2_payload) => {
                            let handshake = internal_state_lock
                                .keys
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
                            internal_state_lock
                                .connected_users
                                .insert(rendezvous2_payload.rendezvous_cookie, handshake);
                        }
                        Payload::Data(data_payload) => {
                            Logger::info(
                                &nickname,
                                format!("Received data from relay {}", sender_id),
                            );
                            let user_handshake = internal_state_lock
                                .connected_users
                                .get(&data_payload.rendezvous_cookie)
                                .unwrap();
                            let decrypted_data =
                                decrypt_buffer_with_aes(user_handshake, &data_payload.data)
                                    .unwrap();
                            Logger::info(
                                &nickname,
                                format!(
                                    "Received String from user with rendezvous cookie {}: {:?}",
                                    data_payload.rendezvous_cookie,
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

                    // send and if it fails, log the error
                    internal_state_lock
                        .events_sender
                        .send(Event(payload_type, sender_id))
                        .unwrap_or_else(|e| {
                            Logger::warn(&nickname, format!("events sender: {}", e))
                        });
                }
                Err(e) => {
                    Logger::error(&nickname, format!("Failed to receive event: {}", e));
                }
            }
        });
    }

    pub fn send_create_to_relay(&self, relay_id: RelayId, circuit_id: CircuitId) -> Result<()> {
        let internal_state_lock = self.internal_state.lock().unwrap();
        let relay_descriptor = Directory::get_relay(relay_id).unwrap();
        Logger::info(
            &self.nickname,
            format!("Sending CREATE payload to: {}", relay_descriptor.nickname,),
        );

        let rsa_public = Rsa::public_key_from_pem(&relay_descriptor.rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = internal_state_lock.keys.dh.public_key().to_vec();
        let aes = generate_random_aes_key();
        let onion_skin =
            OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap()).unwrap();
        let create_payload = Payload::Create(CreatePayload { onion_skin });
        let relay_cell = RelayCell {
            circuit_id,
            payload: serde_json::to_vec(&create_payload)?,
        };
        Communication::send(self.user_descriptor.id, relay_descriptor.id, relay_cell);
        Logger::info(
            &self.nickname,
            format!("Sent CREATE payload to: {}", relay_descriptor.nickname),
        );
        Ok(())
    }

    pub fn send_extend_to_relay(
        &self,
        relay_id: RelayId,
        relay_id_2: RelayId,
        circuit_id: CircuitId,
    ) -> Result<()> {
        let internal_state_lock = self.internal_state.lock().unwrap();
        Logger::info(
            &self.nickname,
            format!(
                "Extending circuit from relay {} to relay {}",
                relay_id, relay_id_2
            ),
        );
        let relay_descriptor = Directory::get_relay(relay_id_2).unwrap();
        let rsa_public = Rsa::public_key_from_pem(&relay_descriptor.rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = internal_state_lock.keys.dh.public_key().to_vec();
        let aes = generate_random_aes_key();
        let onion_skin =
            OnionSkin::new(rsa_public, aes, half_dh_bytes.try_into().unwrap()).unwrap();
        let extend_payload = Payload::Extend(ExtendPayload {
            onion_skin,
            extend_to: relay_descriptor.id,
        });
        let circuit = internal_state_lock.circuits.get(&circuit_id).unwrap();
        let mut handshakes = vec![];
        for relay in circuit {
            handshakes.push(internal_state_lock.handshakes.get(relay).unwrap().clone());
        }
        let mut buffer: Vec<u8> = serde_json::to_vec(&extend_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.nickname,
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
            &self.nickname,
            format!(
                "Sending EXTEND payload to relay {}",
                relay_descriptor.nickname
            ),
        );
        Communication::send(self.user_descriptor.id, relay_id, relay_cell);
        Logger::info(
            &self.nickname,
            format!("Sent EXTEND payload to relay {}", relay_descriptor.nickname),
        );
        Ok(())
    }

    pub fn send_establish_rendezvous_to_relay(
        &self,
        relay_id: RelayId,
        rendezvous_cookie: RendezvousCookieId,
        circuit_id: CircuitId,
    ) -> Result<()> {
        let internal_state_lock = self.internal_state.lock().unwrap();
        Logger::info(
            &self.nickname,
            format!("Sending ESTABLISH_INTRO to relay {}", relay_id),
        );
        let establish_intro_payload =
            Payload::EstablishRendezvous(EstablishRendezvousPayload { rendezvous_cookie });
        let circuit = internal_state_lock.circuits.get(&circuit_id).unwrap();
        let mut handshakes = vec![];
        for relay in circuit {
            handshakes.push(internal_state_lock.handshakes.get(relay).unwrap().clone());
        }
        let mut buffer = serde_json::to_vec(&establish_intro_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.nickname,
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
        Communication::send(self.user_descriptor.id, relay_id, relay_cell);
        Logger::info(
            &self.nickname,
            format!("Sent ESTABLISH_INTRO payload to relay {}", relay_id),
        );
        Ok(())
    }

    pub fn send_establish_introduction_to_relay(
        &self,
        relay_id: RelayId,
        introduction_id: IntroductionPointId,
        circuit_id: CircuitId,
    ) -> Result<()> {
        let internal_state_lock = self.internal_state.lock().unwrap();
        Logger::info(
            &self.nickname,
            format!("Sending ESTABLISH_INTRO to relay {}", relay_id),
        );
        let establish_intro_payload =
            Payload::EstablishIntroduction(EstablishIntroductionPayload {
                introduction_id,
                rsa_publickey: self.user_descriptor.rsa_public.clone(),
            });
        let circuit = internal_state_lock.circuits.get(&circuit_id).unwrap();
        let mut handshakes = vec![];
        for relay in circuit {
            handshakes.push(internal_state_lock.handshakes.get(relay).unwrap().clone());
        }
        let mut buffer = serde_json::to_vec(&establish_intro_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.nickname,
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
        Communication::send(self.user_descriptor.id, relay_id, relay_cell);
        Directory::add_user_introduction_point(
            self.id,
            introduction_id,
            circuit.last().unwrap().clone(),
        );
        Logger::info(
            &self.nickname,
            format!("Sent ESTABLISH_INTRO payload to relay {}", relay_id),
        );
        Ok(())
    }

    pub fn listen_for_event(&self, event: Event) -> Result<()> {
        loop {
            let received_event = self.events_receiver.recv().unwrap();
            if received_event == event {
                return Ok(());
            }
        }
    }

    pub fn send_begin_to_relay(
        &self,
        relay_id: RelayId,
        circuit_id: CircuitId,
        stream_id: StreamId,
        begin_relay_id: RelayId,
    ) -> Result<()> {
        let mut internal_state_lock = self.internal_state.lock().unwrap();
        Logger::info(
            &self.nickname,
            format!("Sending BEGIN to relay {}", relay_id),
        );
        let relay_descriptor = Directory::get_relay(begin_relay_id).unwrap();
        let begin_payload = Payload::Begin(crate::BeginPayload {
            stream_id,
            relay_descriptor,
        });
        let circuit = internal_state_lock.circuits.get(&circuit_id).unwrap();
        let mut handshakes = vec![];
        for relay in circuit {
            handshakes.push(internal_state_lock.handshakes.get(relay).unwrap().clone());
        }
        let mut buffer = serde_json::to_vec(&begin_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.nickname,
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
        Communication::send(self.user_descriptor.id, relay_id, relay_cell);
        internal_state_lock
            .stream_ids
            .insert(stream_id, begin_relay_id);
        Logger::info(
            &self.nickname,
            format!("Sent BEGIN payload to relay at address: {}", relay_id),
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn send_introduce1_to_relay(
        &self,
        relay_id: RelayId,
        introduction_id: IntroductionPointId,
        stream_id: StreamId,
        rendezvous_point_relay_id: RelayId,
        rendezvous_cookie: RendezvousCookieId,
        introduction_rsa_public: Vec<u8>,
        circuit_id: CircuitId,
    ) -> Result<()> {
        let internal_state_lock = self.internal_state.lock().unwrap();
        Logger::info(
            &self.nickname,
            format!("Sending INTRODUCE1 to relay {}", relay_id),
        );
        let relay_descriptor = Directory::get_relay(rendezvous_point_relay_id).unwrap();
        let rsa_public = Rsa::public_key_from_pem(&introduction_rsa_public).unwrap();
        let half_dh_bytes: Vec<u8> = internal_state_lock.keys.dh.public_key().to_vec();
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
        let circuit = internal_state_lock.circuits.get(&circuit_id).unwrap();
        let mut handshakes = vec![];
        for relay in circuit {
            handshakes.push(internal_state_lock.handshakes.get(relay).unwrap().clone());
        }
        let mut buffer = serde_json::to_vec(&introduce1_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.nickname,
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
        Communication::send(self.user_descriptor.id, relay_id, relay_cell);
        Logger::info(
            &self.nickname,
            format!("Sent INTRODUCE1 payload to relay {}", relay_id),
        );
        Ok(())
    }

    pub fn establish_circuit(
        &self,
        circuit_id: CircuitId,
        relay_id_1: RelayId,
        relay_id_2: RelayId,
        relay_id_3: RelayId,
    ) -> Result<()> {
        self.send_create_to_relay(relay_id_1, circuit_id).unwrap();
        self.listen_for_event(Event(PayloadType::Created, relay_id_1))
            .unwrap();
        self.send_extend_to_relay(relay_id_1, relay_id_2, circuit_id)
            .unwrap();
        self.listen_for_event(Event(PayloadType::Extended, relay_id_1))
            .unwrap();
        self.send_extend_to_relay(relay_id_1, relay_id_3, circuit_id)
            .unwrap();
        self.listen_for_event(Event(PayloadType::Extended, relay_id_1))
            .unwrap();
        Logger::info(
            &self.nickname,
            format!(
                "Established a circuit with 3 relays, {} {} {}",
                relay_id_1, relay_id_2, relay_id_3
            ),
        );
        Ok(())
    }

    pub fn send_rendezvous1_to_relay(
        &self,
        relay_id: RelayId,
        rendezvous_cookie: RendezvousCookieId,
        circuit_id: CircuitId,
    ) -> Result<()> {
        let internal_state_lock = self.internal_state.lock().unwrap();
        Logger::info(
            &self.nickname,
            format!("Sending RENDEZVOUS1 to relay {}", relay_id),
        );
        let rendezvous1_payload = Payload::Rendezvous1(crate::Rendezvous1Payload {
            rendezvous_cookie,
            dh_key: internal_state_lock.keys.dh.public_key().to_vec(),
        });
        let circuit = internal_state_lock.circuits.get(&circuit_id).unwrap();
        let mut handshakes = vec![];
        for relay in circuit {
            handshakes.push(internal_state_lock.handshakes.get(relay).unwrap().clone());
        }
        let mut buffer = serde_json::to_vec(&rendezvous1_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.nickname,
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
        Communication::send(self.user_descriptor.id, relay_id, relay_cell);
        Logger::info(
            &self.nickname,
            format!("Sent RENDEZVOUS1 payload to relay {}", relay_id),
        );
        Ok(())
    }

    pub fn send_data_to_relay(
        &self,
        relay_id: RelayId,
        rendezvous_cookie: RendezvousCookieId,
        circuit_id: CircuitId,
        data: Vec<u8>,
    ) -> Result<()> {
        Logger::info(
            &self.nickname,
            format!("Sending DATA to relay at address: {}", relay_id),
        );
        let internal_state_lock = self.internal_state.lock().unwrap();
        let user_handshake = internal_state_lock
            .connected_users
            .get(&rendezvous_cookie)
            .unwrap();
        let encrypted_data = encrypt_buffer_with_aes(user_handshake, &data).unwrap();
        let data_payload: Payload = Payload::Data(crate::DataPayload {
            data: encrypted_data,
            rendezvous_cookie,
        });
        let circuit = internal_state_lock.circuits.get(&circuit_id).unwrap();
        let mut handshakes = vec![];
        for relay in circuit {
            handshakes.push(internal_state_lock.handshakes.get(relay).unwrap().clone());
        }
        let mut buffer = serde_json::to_vec(&data_payload).unwrap();
        for handshake in handshakes.iter().rev() {
            Logger::info(
                &self.nickname,
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
        Communication::send(self.user_descriptor.id, relay_id, relay_cell);
        Logger::info(
            &self.nickname,
            format!("Sent DATA payload to relay {}", relay_id),
        );
        Ok(())
    }
}
