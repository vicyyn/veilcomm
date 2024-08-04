use std::{
    thread::{self, sleep},
    time::Duration,
};
use uuid::Uuid;
use veilcomm2::{Event, PayloadType, Relay, User};

fn main() {
    // let directory = Directory::new(directory_address());
    // directory.start();
    // let api = Api::new();
    // api.start();
    // std::thread::park();
    let mut relay = Relay::new("Relay1".to_string());
    let relay_id = relay.get_relay_descriptor().id;
    thread::spawn(move || {
        relay.start();
    });

    let mut relay_2 = Relay::new("Relay2".to_string());
    let relay_id_2 = relay_2.get_relay_descriptor().id;
    thread::spawn(move || {
        relay_2.start();
    });

    let mut relay_3 = Relay::new("Relay3".to_string());
    let relay_id_3 = relay_3.get_relay_descriptor().id;
    let relay_descriptor_3 = relay_3.get_relay_descriptor();
    thread::spawn(move || {
        relay_3.start();
    });

    let mut relay_4 = Relay::new("Relay4".to_string());
    let relay_id_4 = relay_4.get_relay_descriptor().id;
    thread::spawn(move || {
        relay_4.start();
    });

    let mut relay_5 = Relay::new("Relay5".to_string());
    let relay_id_5 = relay_5.get_relay_descriptor().id;
    thread::spawn(move || {
        relay_5.start();
    });

    let mut relay_6 = Relay::new("Relay6".to_string());
    let relay_id_6 = relay_6.get_relay_descriptor().id;
    let rendezvous_point_descriptor = relay_6.get_relay_descriptor();
    thread::spawn(move || {
        relay_6.start();
    });

    let mut user = User::new("User".to_string());
    let user_2 = User::new("User2".to_string());
    let introduction_id = Uuid::new_v4();
    let rendezvous_cookie = Uuid::new_v4();
    let stream_id = Uuid::new_v4();
    let introduction_rsa_public = user.get_public_rsa_key();

    thread::spawn(move || {
        user.start();
        user.fetch_relays().unwrap();
        let circuit_id = Uuid::new_v4();
        user.establish_circuit(circuit_id, relay_id, relay_id_2, relay_id_3)
            .unwrap();
        user.send_establish_introduction_to_relay(relay_id, introduction_id, circuit_id)
            .unwrap();
        user.listen_for_event(Event(PayloadType::EstablishedIntroduction, relay_id))
            .unwrap();
        user.add_introduction_point(introduction_id, relay_id_3);
        user.update_introduction_points().unwrap();
        user.listen_for_event(Event(PayloadType::Introduce2, relay_id))
            .unwrap();
        let new_circuit_id = Uuid::new_v4();
        user.establish_circuit(new_circuit_id, relay_id_3, relay_id_2, relay_id_6)
            .unwrap();
        user.send_rendezvous1_to_relay(relay_id_3, rendezvous_cookie, new_circuit_id)
            .unwrap();
    });

    sleep(Duration::from_secs(2));
    println!(" * . * . * . *");

    thread::spawn(move || {
        user_2.start();
        user_2.fetch_relays().unwrap();
        let circuit_id = Uuid::new_v4();
        user_2
            .establish_circuit(circuit_id, relay_id_4, relay_id_5, relay_id_6)
            .unwrap();
        user_2
            .send_establish_rendezvous_to_relay(relay_id_4, rendezvous_cookie, circuit_id)
            .unwrap();
        user_2
            .listen_for_event(Event(PayloadType::EstablishedRendezvous, relay_id_4))
            .unwrap();
        user_2
            .send_begin_to_relay(relay_id_4, circuit_id, stream_id, relay_descriptor_3.id)
            .unwrap();
        user_2
            .listen_for_event(Event(PayloadType::Connected, relay_id_4))
            .unwrap();
        user_2
            .send_introduce1_to_relay(
                relay_id_4,
                introduction_id,
                stream_id,
                rendezvous_point_descriptor.id,
                rendezvous_cookie,
                introduction_rsa_public,
                circuit_id,
            )
            .unwrap();
        user_2
            .listen_for_event(Event(PayloadType::IntroduceAck, relay_id_4))
            .unwrap();
        user_2
            .listen_for_event(Event(PayloadType::Rendezvous2, relay_id_4))
            .unwrap();
        let data: Vec<u8> = "Hello, world!".as_bytes().to_vec();
        user_2
            .send_data_to_relay(relay_id_4, rendezvous_cookie, circuit_id, data)
            .unwrap();
    });
    std::thread::park();
}
