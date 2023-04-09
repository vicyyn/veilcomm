use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, RwLock};
use std::thread;

pub mod relay;
pub mod relays;
pub mod user_descriptor;
pub mod user_descriptors;

pub use relay::*;
pub use relays::*;
pub use user_descriptor::*;
pub use user_descriptors::*;

pub fn receive_user_descriptor(stream: TcpStream, user_descriptors: Arc<RwLock<UserDescriptors>>) {
    let mut buffer = [0u8; 1024];
    let mut stream = stream.try_clone().unwrap();
    match stream.read(&mut buffer) {
        Ok(0) => {
            println!(
                "[WARNING] Directory::receive_user_descriptor --> Connection has disconnected from {}",
                stream.peer_addr().unwrap()
            );
        }
        Ok(n) => {
            println!(
                "[SUCCESS] Directory::receive_user_descriptor --> Received : {} bytes from {:?}",
                n,
                stream.peer_addr().unwrap()
            );

            let user_descriptor = UserDescriptor::deserialize(&buffer);
            user_descriptors
                .write()
                .unwrap()
                .add_user_descriptor(user_descriptor);
        }
        Err(e) => {
            println!(
                "[FAILED] Directory::receive_user_descriptor --> Error reading from socket: {}",
                e
            );
        }
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
    let relays = relays.read().unwrap();
    println!(
        "[SUCCESS] Directory::send_relays --> sent {} relays",
        relays.len()
    );
    let mut stream = stream.try_clone().unwrap();
    stream.write(&relays.serialize()).unwrap();
}

pub fn send_user_descriptors(stream: TcpStream, user_descriptors: Arc<RwLock<UserDescriptors>>) {
    let user_descriptors = user_descriptors.read().unwrap();
    println!(
        "[SUCCESS] Directory::send_relays --> sent {} relays",
        user_descriptors.len()
    );
    let mut stream = stream.try_clone().unwrap();
    stream.write(&user_descriptors.serialize()).unwrap();
}

pub fn start_directory(address: SocketAddr) {
    thread::spawn(move || {
        let relays = Arc::new(RwLock::new(Relays::new()));
        let user_descriptors = Arc::new(RwLock::new(UserDescriptors::new()));
        let socket = TcpListener::bind(address)
            .expect("[FAILED] Directory::start_directory --> Error while binding TcpSocket to specified addr");

        loop {
            match socket.accept() {
                Ok((stream, addr)) => {
                    println!(
                        "[SUCCESS] Directory::start_directory - New client connected: {:?}",
                        addr
                    );

                    thread::spawn({
                        let cloned_relays = Arc::clone(&relays);
                        let cloned_user_descriptors = Arc::clone(&user_descriptors);

                        move || {
                            receive_relay(stream.try_clone().unwrap(), cloned_relays.clone());
                            send_relays(stream.try_clone().unwrap(), cloned_relays);
                            // send_user_descriptors(
                            //     stream.try_clone().unwrap(),
                            //     cloned_user_descriptors.clone(),
                            // );
                            receive_user_descriptor(stream, cloned_user_descriptors);
                        }
                    });
                }
                Err(e) => {
                    println!(
                    "[FAILED] Directory::start_directory - Error accepting client connection: {}",
                    e
                );
                }
            }
        }
    });
}

pub fn new_socket_addr(port: u16) -> SocketAddr {
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
}
