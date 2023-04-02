use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::{
    env,
    net::{SocketAddr, TcpListener},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Relays(Vec<Relay>);

impl Relays {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_relay(&mut self, relay: Relay) {
        self.0.push(relay);
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Relays::serialize --> Unable to serialize")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Relays::deserialize --> Unable to deserialize")
    }
}

#[derive(Serialize, Deserialize)]
pub struct Relay {
    pub nickname: String,
    pub identity_key: String,
    pub address: SocketAddr,
    pub contact_information: String,
}

impl Relay {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Relay::serialize --> Unable to serialize")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Relay::deserialize --> Unable to deserialize")
    }
}

pub fn receive_relay(stream: TcpStream, relays: Arc<RwLock<Relays>>) {
    let mut buffer = [0u8; 1024];
    let mut stream = stream.try_clone().unwrap();
    match stream.read(&mut buffer) {
        Ok(0) => {
            println!(
                "[WARNING] Directory::receive --> Connection has disconnected from {}",
                stream.peer_addr().unwrap()
            );
        }
        Ok(n) => {
            println!(
                "[SUCCESS] Directory::receive --> Received : {} bytes from {:?}",
                n,
                stream.peer_addr().unwrap()
            );

            let relay = Relay::deserialize(&buffer);
            relays.write().unwrap().add_relay(relay);
        }
        Err(e) => {
            println!(
                "[FAILED] Directory::receive --> Error reading from socket: {}",
                e
            );
        }
    }
}

pub fn send_relays(stream: TcpStream, relays: Arc<RwLock<Relays>>) {
    let mut stream = stream.try_clone().unwrap();
    stream.write(&relays.read().unwrap().serialize()).unwrap();
}

fn main() {
    let relays = Arc::new(RwLock::new(Relays::new()));
    let args: Vec<String> = env::args().collect();
    let address = SocketAddr::new(
        std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        args[1].parse().unwrap(),
    );

    let socket = TcpListener::bind(address).expect(
        "[FAILED] tor::listen_for_connections --> Error while binding TcpSocket to specified addr",
    );

    loop {
        match socket.accept() {
            Ok((stream, addr)) => {
                println!(
                    "[SUCCESS] tor::listen_for_connections - New client connected: {:?}",
                    addr
                );

                thread::spawn({
                    let cloned_relays = Arc::clone(&relays);
                    move || {
                        receive_relay(stream.try_clone().unwrap(), cloned_relays.clone());
                        send_relays(stream, cloned_relays);
                    }
                });
            }
            Err(e) => {
                println!(
                    "[FAILED] tor::listen_for_connections - Error accepting client connection: {}",
                    e
                );
            }
        }
    }
}
