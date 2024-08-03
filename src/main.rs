mod api;
use api::Api;
use simple_logger::SimpleLogger;
use veilcomm2::{directory_address, Directory};

#[tokio::main]
async fn main() {
    SimpleLogger::with_level(SimpleLogger::new(), log::LevelFilter::Info)
        .init()
        .unwrap();

    let directory = Directory::new(directory_address());
    directory.start();
    let api = Api::new();
    api.start();
    std::thread::park();
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_logger::SimpleLogger;
    use std::{net::SocketAddr, str::FromStr, thread::sleep, time::Duration};
    use uuid::Uuid;
    use veilcomm2::{Event, PayloadType, Relay, User};

    #[tokio::test]
    async fn test_main() {
        SimpleLogger::with_level(SimpleLogger::new(), log::LevelFilter::Info)
            .init()
            .unwrap();

        let directory = Directory::new(directory_address());
        directory.start();
        sleep(Duration::from_secs(2));

        let relay_address = SocketAddr::from_str("127.0.0.1:3031").unwrap();
        let relay = Relay::new(relay_address, "Relay1".to_string());

        relay.start().await.unwrap();

        let relay_address_2 = SocketAddr::from_str("127.0.0.1:3032").unwrap();
        let relay_2 = Relay::new(relay_address_2, "Relay2".to_string());

        relay_2.start().await.unwrap();

        let relay_address_3 = SocketAddr::from_str("127.0.0.1:3033").unwrap();
        let relay_3 = Relay::new(relay_address_3, "Relay3".to_string());
        let relay_descriptor_3 = relay_3.get_relay_descriptor();

        relay_3.start().await.unwrap();

        let relay_address_4 = SocketAddr::from_str("127.0.0.1:3034").unwrap();
        let relay_4 = Relay::new(relay_address_4, "Relay4".to_string());

        relay_4.start().await.unwrap();

        let relay_address_5 = SocketAddr::from_str("127.0.0.1:3035").unwrap();
        let relay_5 = Relay::new(relay_address_5, "Relay5".to_string());

        relay_5.start().await.unwrap();

        let relay_address_6 = SocketAddr::from_str("127.0.0.1:3036").unwrap();
        let relay_6 = Relay::new(relay_address_6, "Relay6".to_string());
        let rendezvous_point_descriptor = relay_6.get_relay_descriptor();

        relay_6.start().await.unwrap();

        let mut user = User::new("User".to_string());
        let user_2 = User::new("User2".to_string());
        let introduction_id = Uuid::new_v4();
        let rendezvous_cookie = Uuid::new_v4();
        let stream_id = Uuid::new_v4();
        let introduction_rsa_public = user.user_descriptor.rsa_public.clone();

        tokio::spawn(async move {
            user.start().await.unwrap();
            user.fetch_relays().await.unwrap();
            let circuit_id = Uuid::new_v4();
            user.establish_circuit(circuit_id, relay_address, relay_address_2, relay_address_3)
                .await
                .unwrap();
            user.send_establish_introduction_to_relay(relay_address, introduction_id, circuit_id)
                .await
                .unwrap();
            user.listen_for_event(Event(PayloadType::EstablishedIntroduction, relay_address))
                .await
                .unwrap();
            user.add_introduction_point(introduction_id, relay_address_3);
            user.update_introduction_points().await.unwrap();
            user.listen_for_event(Event(PayloadType::Introduce2, relay_address))
                .await
                .unwrap();
            let new_circuit_id = Uuid::new_v4();
            user.establish_circuit(
                new_circuit_id,
                relay_address_3,
                relay_address_2,
                relay_address_6,
            )
            .await
            .unwrap();
            user.send_rendezvous1_to_relay(relay_address_3, rendezvous_cookie, new_circuit_id)
                .await
                .unwrap();
        });

        sleep(Duration::from_secs(2));
        println!(" * . * . * . *");

        tokio::spawn(async move {
            user_2.start().await.unwrap();
            user_2.fetch_relays().await.unwrap();
            let circuit_id = Uuid::new_v4();
            user_2
                .establish_circuit(
                    circuit_id,
                    relay_address_4,
                    relay_address_5,
                    relay_address_6,
                )
                .await
                .unwrap();
            user_2
                .send_establish_rendezvous_to_relay(relay_address_4, rendezvous_cookie, circuit_id)
                .await
                .unwrap();
            user_2
                .listen_for_event(Event(PayloadType::EstablishedRendezvous, relay_address_4))
                .await
                .unwrap();
            user_2
                .send_begin_to_relay(
                    relay_address_4,
                    circuit_id,
                    stream_id,
                    relay_descriptor_3.address,
                )
                .await
                .unwrap();
            user_2
                .listen_for_event(Event(PayloadType::Connected, relay_address_4))
                .await
                .unwrap();
            user_2
                .send_introduce1_to_relay(
                    relay_address_4,
                    introduction_id,
                    stream_id,
                    rendezvous_point_descriptor.address,
                    rendezvous_cookie,
                    introduction_rsa_public,
                    circuit_id,
                )
                .await
                .unwrap();
            user_2
                .listen_for_event(Event(PayloadType::IntroduceAck, relay_address_4))
                .await
                .unwrap();
            user_2
                .listen_for_event(Event(PayloadType::Rendezvous2, relay_address_4))
                .await
                .unwrap();
            let data: Vec<u8> = "Hello, world!".as_bytes().to_vec();
            user_2
                .send_data_to_relay(relay_address_4, rendezvous_cookie, circuit_id, data)
                .await
                .unwrap();
        });
        std::thread::park();
    }
}
