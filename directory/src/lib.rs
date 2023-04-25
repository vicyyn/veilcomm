use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, RwLock};
use std::thread;

pub mod directory_event;
pub mod relay;
pub mod relays;
pub mod user_descriptor;
pub mod user_descriptors;

pub use directory_event::*;
pub use relay::*;
pub use relays::*;
pub use user_descriptor::*;
pub use user_descriptors::*;

pub fn listen_for_events(
    stream: TcpStream,
    relays: Arc<RwLock<Relays>>,
    user_descriptors: Arc<RwLock<UserDescriptors>>,
) {
    loop {
        let mut buffer = [0u8; 16384]; // 16KB
        let mut stream = stream.try_clone().unwrap();
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!(
                    "[WARNING] Directory::listen_for_events --> Connection has disconnected from {}",
                    stream.peer_addr().unwrap()
                );
                break;
            }
            Ok(n) => {
                println!(
                    "[SUCCESS] Directory::listen_for_events --> Received : {} bytes from {:?}",
                    n,
                    stream.peer_addr().unwrap()
                );
                match DirectoryEvent::deserialize(buffer[0]) {
                    DirectoryEvent::AddRelay => {
                        let relay = Relay::deserialize(&buffer[1..n]);
                        relays.write().unwrap().add_relay(relay);
                        stream
                            .write(&[DirectoryEvent::AddedRelay.serialize()])
                            .unwrap();
                    }
                    DirectoryEvent::AddUserDescriptor => {
                        let user_descriptor = UserDescriptor::deserialize(&buffer[1..n]);
                        user_descriptors
                            .write()
                            .unwrap()
                            .add_user_descriptor(user_descriptor);
                        stream
                            .write(&[DirectoryEvent::AddedUserDescriptor.serialize()])
                            .unwrap();
                    }
                    DirectoryEvent::GetRelays => {
                        let relays = relays.read().unwrap();
                        println!(
                            "[SUCCESS] Directory::send_relays --> sent {} relays",
                            relays.len()
                        );
                        let mut stream = stream.try_clone().unwrap();
                        stream.write(&relays.serialize()).unwrap();
                    }
                    DirectoryEvent::GetUserDescriptors => {
                        let user_descriptors = user_descriptors.read().unwrap();
                        println!("[SUCCESS] Directory::send_user_descriptors --> sent {} user_descriptors",user_descriptors.len());
                        let mut stream = stream.try_clone().unwrap();
                        stream.write(&user_descriptors.serialize()).unwrap();
                    }
                    _ => {
                        println!(
                            "[FAILED] Directory::listen_for_events --> Invalid DirectoryEvent"
                        );
                    }
                }
            }
            Err(e) => {
                println!(
                    "[FAILED] Directory::listen_for_events --> Error reading from socket: {}",
                    e
                );
                break;
            }
        }
    }
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
                        move || listen_for_events(stream, cloned_relays, cloned_user_descriptors)
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
