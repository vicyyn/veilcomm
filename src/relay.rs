use crate::{
    decrypt_buffer_with_aes, encrypt_buffer_with_aes, get_handshake_from_onion_skin,
    payloads::{self, CreatePayload},
    Communication, ConnectedPayload, Keys, Payload, PayloadType, RelayCell,
};
use crate::{Directory, Logger};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RelayDescriptor {
    pub id: Uuid,
    pub nickname: String,
    pub rsa_public: Vec<u8>,
}

pub struct Relay {
    relay_descriptor: RelayDescriptor,
    handshakes: HashMap<Uuid, Vec<u8>>,
    keys: Keys,
    circuits_ids: HashMap<Uuid, Uuid>,
    // bool is direction, if true then we have to decrypt, if false then we have to encrypt
    circuits_map: HashMap<Uuid, (Uuid, bool)>,
    rendezvous_points: HashMap<Uuid, Uuid>,
    introduction_points: HashMap<Uuid, Uuid>,
    streams: HashMap<Uuid, Uuid>,
}

impl Relay {
    pub fn get_relay_descriptor(&self) -> RelayDescriptor {
        self.relay_descriptor.clone()
    }

    pub fn new(nickname: String) -> Self {
        Logger::info(&nickname, "Creating new relay");
        let rsa = openssl::rsa::Rsa::generate(2048).unwrap();
        Self {
            relay_descriptor: RelayDescriptor {
                id: Uuid::new_v4(),
                nickname: nickname.clone(),
                rsa_public: rsa.public_key_to_pem().unwrap(),
            },
            keys: Keys {
                rsa_private: rsa,
                dh: openssl::dh::Dh::get_2048_256()
                    .unwrap()
                    .generate_key()
                    .unwrap(),
            },
            handshakes: HashMap::new(),
            circuits_ids: HashMap::new(),
            circuits_map: HashMap::new(),
            rendezvous_points: HashMap::new(),
            introduction_points: HashMap::new(),
            streams: HashMap::new(),
        }
    }

    /// Start the relay server
    pub fn start(&mut self) {
        Logger::info(&self.relay_descriptor.nickname, "Starting the relay server");

        Logger::info(
            &self.relay_descriptor.nickname,
            "Registering relay with directory server",
        );
        Directory::publish_relay(self.relay_descriptor.clone());

        Logger::info(&self.relay_descriptor.nickname, "Registration successful");
        Logger::info(
            &self.relay_descriptor.nickname,
            "Registering with communication server",
        );
        let receiver = Communication::register(self.relay_descriptor.id);
        Logger::info(
            &self.relay_descriptor.nickname,
            "Successfully registered with the communication server",
        );

        let nickname = self.relay_descriptor.nickname.clone();
        let my_id = self.relay_descriptor.id;

        loop {
            match receiver.recv() {
                Ok((sender_id, relay_cell)) => {
                    Logger::info(&nickname, "Received relay cell");

                    if let Some((next_circuit_id, direction)) =
                        self.circuits_map.get(&relay_cell.circuit_id)
                    {
                        if *direction {
                            // decrypt with handshake then forward to next relay
                            let handshake = self.handshakes.get(&relay_cell.circuit_id).unwrap();
                            let decrypted_payload =
                                decrypt_buffer_with_aes(&handshake[0..32], &relay_cell.payload)
                                    .unwrap();
                            if let Ok(payload) =
                                serde_json::from_slice::<Payload>(&decrypted_payload)
                            {
                                if payload.get_type() == PayloadType::Data {
                                    let id = self.circuits_ids.get(next_circuit_id).unwrap();
                                    let handshake = self
                                        .handshakes
                                        .get(next_circuit_id)
                                        .expect("Handshake not found");
                                    let encrypted_payload =
                                        encrypt_buffer_with_aes(handshake, &decrypted_payload)
                                            .unwrap();
                                    let relay_cell = RelayCell {
                                        circuit_id: *next_circuit_id,
                                        payload: encrypted_payload,
                                    };
                                    Logger::info(&nickname, "forwarding data payload");
                                    Communication::send(my_id, *id, relay_cell);
                                    Logger::info(&nickname, "forwarded data payload");
                                } else {
                                    Logger::error(
                                        &nickname,
                                        &format!(
                                            "expected data payload, got {:?} payload",
                                            payload.get_type()
                                        ),
                                    );
                                }
                            } else {
                                let relay_cell = RelayCell {
                                    circuit_id: *next_circuit_id,
                                    payload: decrypted_payload,
                                };
                                let id = self.circuits_ids.get(next_circuit_id).unwrap();
                                Logger::info(&nickname, "forwarding relay cell to next relay");
                                Communication::send(my_id, *id, relay_cell);
                                Logger::info(
                                    &nickname,
                                    &"Forwarded relay cell to next relay".to_string(),
                                );
                            }
                            continue;
                        }
                    };

                    // get the payload
                    let payload = if let Some(handshake) =
                        self.handshakes.get(&relay_cell.circuit_id)
                    {
                        let decrypted_payload =
                            decrypt_buffer_with_aes(&handshake[0..32], &relay_cell.payload)
                                .unwrap();
                        Logger::info(
                            &nickname,
                            &format!(
                                "Decrypted payload with handshake for circuit {}",
                                relay_cell.circuit_id
                            ),
                        );
                        serde_json::from_slice::<Payload>(&decrypted_payload).unwrap()
                    } else {
                        Logger::info(
                            &nickname,
                            &format!("No handshake found for circuit {}", relay_cell.circuit_id),
                        );
                        if let Ok(payload) = serde_json::from_slice::<Payload>(&relay_cell.payload)
                        {
                            payload
                        } else if let Some((next_circuit_id, direction)) =
                            self.circuits_map.get(&relay_cell.circuit_id)
                        {
                            if !*direction {
                                let id = self.circuits_ids.get(next_circuit_id).unwrap();
                                Logger::info(
                                    &nickname,
                                    &format!(
                                        "Forwarding payload back to circuit {}",
                                        next_circuit_id
                                    ),
                                );
                                let handshake = self.handshakes.get(next_circuit_id).unwrap();
                                let encrypted_payload =
                                    encrypt_buffer_with_aes(handshake, &relay_cell.payload)
                                        .unwrap();
                                let relay_cell = RelayCell {
                                    circuit_id: *next_circuit_id,
                                    payload: encrypted_payload,
                                };
                                Communication::send(my_id, *id, relay_cell);
                                Logger::info(
                                    &nickname,
                                    &"Forwarded payload to previous relay".to_string(),
                                );
                            } else {
                                Logger::error(&nickname, &format!("direction is wrong, expected false, got true for circuit {} coming from {}",
                                        relay_cell.circuit_id,
                                        sender_id
                                    ));
                            }
                            continue;
                        } else {
                            Logger::error(
                                &nickname,
                                &format!(
                                    "no circuit found for circuit {} coming from {}",
                                    relay_cell.circuit_id, sender_id
                                ),
                            );
                            continue;
                        }
                    };

                    Logger::info(
                        &nickname,
                        &format!("Received payload: {:?}", payload.get_type()),
                    );

                    match payload {
                        Payload::Create(create_payload) => {
                            let handshake = get_handshake_from_onion_skin(
                                create_payload.onion_skin,
                                &self.keys.dh,
                                &self.keys.rsa_private,
                            )
                            .unwrap();

                            self.handshakes.insert(relay_cell.circuit_id, handshake);
                            Logger::info(
                                &nickname,
                                format!(
                                    "Adding a new circuit with ID: {}",
                                    relay_cell.circuit_id.clone()
                                ),
                            );

                            if self
                                .circuits_ids
                                .insert(relay_cell.circuit_id, sender_id)
                                .is_some()
                            {
                                Logger::error(&nickname, "Circuit ID already exists".to_string());
                                continue;
                            }

                            Logger::info(&nickname, "Sending created payload".to_string());
                            let created_payload = Payload::Created(payloads::CreatedPayload {
                                dh_key: self.keys.dh.public_key().to_vec(),
                            });
                            let relay_cell = RelayCell {
                                circuit_id: relay_cell.circuit_id,
                                payload: serde_json::to_vec(&created_payload).unwrap(),
                            };

                            Communication::send(my_id, sender_id, relay_cell);
                            Logger::info(&nickname, &"Sent created payload".to_string());
                        }
                        Payload::Created(created_payload) => {
                            if let Some((next_circuit_id, direction)) =
                                self.circuits_map.get(&relay_cell.circuit_id)
                            {
                                if !*direction {
                                    let id = self.circuits_ids.get(next_circuit_id).unwrap();
                                    Logger::info(
                                        &nickname,
                                        format!(
                                            "Forwarding extended payload back to circuit {}",
                                            next_circuit_id
                                        ),
                                    );
                                    let extended_payload =
                                        Payload::Extended(payloads::ExtendedPayload {
                                            extend_to: sender_id,
                                            dh_key: created_payload.dh_key,
                                        });
                                    let handshake = self.handshakes.get(next_circuit_id).unwrap();
                                    let encrypted_payload = encrypt_buffer_with_aes(
                                        handshake,
                                        &serde_json::to_vec(&extended_payload).unwrap(),
                                    )
                                    .unwrap();
                                    let relay_cell = RelayCell {
                                        circuit_id: *next_circuit_id,
                                        payload: encrypted_payload,
                                    };
                                    Communication::send(my_id, *id, relay_cell);
                                    Logger::info(
                                        &nickname,
                                        "Forwarded payload to previous relay".to_string(),
                                    );
                                } else {
                                    Logger::error(&nickname, format!("direction is wrong, expected false, got true for circuit {} coming from {}",
                                        relay_cell.circuit_id,
                                        sender_id
                                    ));
                                }
                            }
                        }
                        Payload::Extend(extend_payload) => {
                            let id = extend_payload.extend_to;
                            // Check if the circuit is already extended
                            if self.circuits_map.get(&relay_cell.circuit_id).is_some() {
                                Logger::error(&nickname, "Circuit already extended".to_string());
                                continue;
                            }
                            Logger::info(
                                &nickname,
                                format!("Extending circuit with ID: {}", relay_cell.circuit_id),
                            );
                            let new_circuit_id = Uuid::new_v4();
                            self.circuits_map
                                .insert(relay_cell.circuit_id, (new_circuit_id, true));
                            self.circuits_map
                                .insert(new_circuit_id, (relay_cell.circuit_id, false));
                            self.circuits_ids.insert(new_circuit_id, id);

                            // forward the extend payload to the next relay as create payload
                            let create_payload = Payload::Create(CreatePayload {
                                onion_skin: extend_payload.onion_skin,
                            });
                            let relay_cell = RelayCell {
                                circuit_id: new_circuit_id,
                                payload: serde_json::to_vec(&create_payload)
                                    .expect("Failed to serialize JSON"),
                            };
                            Communication::send(my_id, id, relay_cell);
                        }
                        Payload::EstablishRendezvous(establish_rendezvous) => {
                            let rendezvous_cookie = establish_rendezvous.rendezvous_cookie;
                            self.rendezvous_points
                                .insert(rendezvous_cookie, relay_cell.circuit_id);
                            let established_rendezvous_payload = Payload::EstablishedRendezvous(
                                payloads::EstablishedRendezvousPayload {},
                            );
                            let handshake = self
                                .handshakes
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
                            Communication::send(my_id, sender_id, relay_cell);
                            Logger::info(
                                &nickname,
                                &format!("Established rendezvous, cookie: {}", rendezvous_cookie),
                            );
                        }
                        Payload::EstablishIntroduction(establish_introduction) => {
                            let introduction_id = establish_introduction.introduction_id;
                            self.introduction_points
                                .insert(introduction_id, relay_cell.circuit_id);
                            let established_introduction_payload = Payload::EstablishedIntroduction(
                                payloads::EstablishedIntroductionPayload {},
                            );
                            let handshake = self
                                .handshakes
                                .get(&relay_cell.circuit_id)
                                .expect("Handshake not found");
                            let encrypted_payload = encrypt_buffer_with_aes(
                                handshake,
                                &serde_json::to_vec(&established_introduction_payload)
                                    .expect("Failed to serialize JSON"),
                            )
                            .unwrap();
                            let relay_cell = RelayCell {
                                circuit_id: relay_cell.circuit_id,
                                payload: encrypted_payload,
                            };
                            Communication::send(my_id, sender_id, relay_cell);
                            Logger::info(
                                &nickname,
                                &format!("Established introduction, id: {}", introduction_id),
                            );
                        }
                        Payload::Begin(begin_payload) => {
                            let connected_payload = Payload::Connected(ConnectedPayload {});
                            let handshake = self.handshakes.get(&relay_cell.circuit_id).unwrap();
                            let encrypted_payload = encrypt_buffer_with_aes(
                                handshake,
                                &serde_json::to_vec(&connected_payload).unwrap(),
                            )
                            .unwrap();
                            let begin_relay_cell = RelayCell {
                                circuit_id: relay_cell.circuit_id,
                                payload: encrypted_payload,
                            };
                            self.streams
                                .insert(begin_payload.stream_id, begin_payload.relay_descriptor.id);
                            Communication::send(my_id, sender_id, begin_relay_cell);
                        }
                        Payload::Introduce1(introduce1_payload) => {
                            // verify that introduction id matches and that the stream exists
                            let stream_id = introduce1_payload.stream_id;
                            let introduction_id = introduce1_payload.introduction_id;

                            if let Some(id) = self.streams.get(&stream_id) {
                                Logger::info(&nickname, "Stream found");
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
                                Communication::send(my_id, *id, relay_cell.clone());
                                Logger::info(
                                    &nickname,
                                    format!("Sent introduce1 payload to stream {}", stream_id),
                                );

                                Logger::info(&nickname, "Sending introduce ack payload");

                                let introduce_ack_payload =
                                    Payload::IntroduceAck(payloads::IntroduceAckPayload {});
                                let handshake = self
                                    .handshakes
                                    .get(&relay_cell.circuit_id)
                                    .expect("Handshake not found");
                                let encrypted_payload = encrypt_buffer_with_aes(
                                    handshake,
                                    &serde_json::to_vec(&introduce_ack_payload).unwrap(),
                                )
                                .unwrap();
                                let relay_cell = RelayCell {
                                    circuit_id: relay_cell.circuit_id,
                                    payload: encrypted_payload,
                                };
                                Communication::send(my_id, sender_id, relay_cell);
                                Logger::info(&nickname, "Sent introduce ack payload");
                            } else {
                                Logger::warn(&nickname, "Stream not found");
                                if let Some(introduction_circuit_id) =
                                    self.introduction_points.get(&introduction_id)
                                {
                                    Logger::info(&nickname, "Introduction point found");
                                    let introduction_relay_id = self
                                        .circuits_ids
                                        .get(introduction_circuit_id)
                                        .expect("Introduction point not found");
                                    Logger::info(
                                        &nickname,
                                        format!(
                                            "Sending introduce2 payload to introduction relay {}",
                                            introduction_relay_id
                                        ),
                                    );
                                    let introduce2_payload =
                                        Payload::Introduce2(payloads::Introduce2Payload {
                                            rendezvous_point_descriptor: introduce1_payload
                                                .rendezvous_point_descriptor,
                                            rendezvous_cookie: introduce1_payload.rendezvous_cookie,
                                            onion_skin: introduce1_payload.onion_skin,
                                        });
                                    let handshake = self
                                        .handshakes
                                        .get(introduction_circuit_id)
                                        .expect("Handshake not found");
                                    let introduce2_payload = encrypt_buffer_with_aes(
                                        handshake,
                                        &serde_json::to_vec(&introduce2_payload).unwrap(),
                                    )
                                    .unwrap();
                                    let relay_cell = RelayCell {
                                        circuit_id: *introduction_circuit_id,
                                        payload: introduce2_payload,
                                    };
                                    Communication::send(my_id, *introduction_relay_id, relay_cell);
                                } else {
                                    Logger::error(&nickname, "Introduction point not found");
                                    continue;
                                }
                            }
                        }
                        Payload::Rendezvous1(rendezvous1_payload) => {
                            // connect the two circuits together
                            if let Some(original_circuit_id) = self
                                .rendezvous_points
                                .get(&rendezvous1_payload.rendezvous_cookie)
                            {
                                self.circuits_map
                                    .insert(relay_cell.circuit_id, (*original_circuit_id, false));
                                self.circuits_map
                                    .insert(*original_circuit_id, (relay_cell.circuit_id, true));
                                let rendezvous2_payload =
                                    Payload::Rendezvous2(payloads::Rendezvous2Payload {
                                        rendezvous_cookie: rendezvous1_payload.rendezvous_cookie,
                                        dh_key: rendezvous1_payload.dh_key,
                                    });
                                let handshake = self
                                    .handshakes
                                    .get(original_circuit_id)
                                    .expect("Handshake not found");
                                let encrypted_payload = encrypt_buffer_with_aes(
                                    handshake,
                                    &serde_json::to_vec(&rendezvous2_payload).unwrap(),
                                )
                                .unwrap();
                                let relay_cell = RelayCell {
                                    circuit_id: *original_circuit_id,
                                    payload: encrypted_payload,
                                };
                                let id = self
                                    .circuits_ids
                                    .get(original_circuit_id)
                                    .expect("Original circuit not found");
                                Communication::send(my_id, *id, relay_cell);
                            } else {
                                Logger::error(&nickname, "Rendezvous point not found");
                                continue;
                            }
                        }
                        Payload::Data(_) => {
                            let circuit_id =
                                self.circuits_map.get(&relay_cell.circuit_id).unwrap().0;
                            let handshake = self
                                .handshakes
                                .get(&circuit_id)
                                .expect("Handshake not found");
                            let encrypted_payload = encrypt_buffer_with_aes(
                                handshake,
                                &serde_json::to_vec(&payload).unwrap(),
                            )
                            .unwrap();
                            let relay_cell = RelayCell {
                                circuit_id,
                                payload: encrypted_payload,
                            };
                            let id = self.circuits_ids.get(&circuit_id).unwrap();
                            Communication::send(my_id, *id, relay_cell);
                            Logger::info(&nickname, "Forwarded data payload");
                        }
                        _ => {
                            Logger::error(&nickname, &"Unhandled payload type".to_string());
                        }
                    }
                }
                Err(e) => {
                    Logger::error(&nickname, format!("Failed to read from socket: {}", e));
                    continue;
                }
            }
        }
    }
}
