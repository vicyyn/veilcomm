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

pub fn fetch_relays(stream: TcpStream) -> Option<Relays> {
    let mut stream = stream.try_clone().unwrap();
    stream
        .write(&[DirectoryEvent::GetRelays.serialize()])
        .unwrap();
    let mut buffer = [0u8; 16384]; // 16KB
    match stream.read(&mut buffer) {
        Ok(0) => {
            println!(
                "[WARNING] Directory::fetch_relays --> Connection has disconnected from {}",
                stream.peer_addr().unwrap()
            );
            None
        }
        Ok(n) => {
            println!(
                "[SUCCESS] Directory::fetch_relays --> Received : {} bytes from {:?}",
                n,
                stream.peer_addr().unwrap()
            );
            Some(Relays::deserialize(&buffer[0..n]))
        }
        Err(e) => {
            println!(
                "[FAILED] Directory::fetch_relays --> Error reading from socket: {}",
                e
            );
            None
        }
    }
}

pub fn fetch_user_descriptors(stream: TcpStream) -> Option<UserDescriptors> {
    let mut stream = stream.try_clone().unwrap();
    stream
        .write(&[DirectoryEvent::GetUserDescriptors.serialize()])
        .unwrap();
    let mut buffer = [0u8; 16384]; // 16KB
    match stream.read(&mut buffer) {
        Ok(0) => {
            println!(
                "[WARNING] Directory::fetch_user_descriptors --> Connection has disconnected from {}",
                stream.peer_addr().unwrap()
            );
            None
        }
        Ok(n) => {
            println!(
                "[SUCCESS] Directory::fetch_user_descriptors --> Received : {} bytes from {:?}",
                n,
                stream.peer_addr().unwrap()
            );
            Some(UserDescriptors::deserialize(&buffer[0..n]))
        }
        Err(e) => {
            println!(
                "[FAILED] Directory::fetch_user_descriptors --> Error reading from socket: {}",
                e
            );
            None
        }
    }
}

pub fn publish_user_descriptor(stream: TcpStream, user_descriptor: UserDescriptor) {
    let mut request_buf = vec![];
    let mut buffer = [0u8; 16384]; // 16KB
    request_buf.push(DirectoryEvent::AddUserDescriptor.serialize());
    request_buf.extend(user_descriptor.serialize());
    stream.try_clone().unwrap().write(&request_buf).unwrap();
    request_buf.clear();
    let len = stream.try_clone().unwrap().read(&mut buffer).unwrap();
    assert!(len == 1);
    assert!(buffer[0] == DirectoryEvent::AddedUserDescriptor.serialize());
}

pub fn publish_relay(stream: TcpStream, relay: Relay) {
    let mut request_buf = vec![];
    let mut buffer = [0u8; 16384]; // 16KB
    request_buf.push(DirectoryEvent::AddRelay.serialize());
    request_buf.extend(relay.serialize());
    stream.try_clone().unwrap().write(&request_buf).unwrap();
    request_buf.clear();
    let len = stream.try_clone().unwrap().read(&mut buffer).unwrap();
    assert!(len == 1);
    assert!(buffer[0] == DirectoryEvent::AddedRelay.serialize());
}

pub fn listen_for_events(stream: TcpStream, relays: Relays, user_descriptors: UserDescriptors) {
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
                        println!(
                            "[SUCCESS] Directory::listen_for_events --> Received Add Relay event"
                        );
                        let relay = Relay::deserialize(&buffer[1..n]);
                        relays.add_relay(relay);
                        stream
                            .write(&[DirectoryEvent::AddedRelay.serialize()])
                            .unwrap();
                    }
                    DirectoryEvent::AddUserDescriptor => {
                        println!(
                            "[SUCCESS] Directory::listen_for_events --> Received Add User Descriptor event");
                        let user_descriptor = UserDescriptor::deserialize(&buffer[1..n]);
                        user_descriptors.add_user_descriptor(user_descriptor);
                        stream
                            .write(&[DirectoryEvent::AddedUserDescriptor.serialize()])
                            .unwrap();
                    }
                    DirectoryEvent::GetRelays => {
                        println!(
                            "[SUCCESS] Directory::send_relays --> sent {} relays",
                            relays.len()
                        );
                        let mut stream = stream.try_clone().unwrap();
                        stream.write(&relays.serialize()).unwrap();
                    }
                    DirectoryEvent::GetUserDescriptors => {
                        println!("[SUCCESS] Directory::send_user_descriptors --> sent {} user_descriptors",user_descriptors.len());
                        let mut stream = stream.try_clone().unwrap();
                        stream.write(&user_descriptors.serialize()).unwrap();
                    }
                    _ => {
                        println!(
                            "[FAILED] Directory::listen_for_events --> Invalid Directory Event"
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
    let relays = Relays::new();
    let user_descriptors = UserDescriptors::new();
    let socket = TcpListener::bind(address).expect(
        "[FAILED] Directory::start_directory --> Error while binding TcpSocket to specified addr",
    );

    loop {
        match socket.accept() {
            Ok((stream, addr)) => {
                println!(
                    "[SUCCESS] Directory::start_directory - New client connected: {:?}",
                    addr
                );

                thread::spawn({
                    let cloned_relays = relays.clone();
                    let cloned_user_descriptors = user_descriptors.clone();
                    move || {
                        listen_for_events(stream, cloned_relays, cloned_user_descriptors);
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
}

pub fn connect_to_directory(relay: Relay, address: SocketAddr) -> Option<TcpStream> {
    match TcpStream::connect(address) {
        Ok(stream) => {
            println!(
                "[SUCCESS] tor::connect_to_directory --> Connected to Directory: {:?}",
                address
            );
            publish_relay(stream.try_clone().unwrap(), relay);
            Some(stream)
        }
        Err(e) => {
            println!(
                "[FAILED] tor::connect_to_peer --> Error Connecting to Peer: {}",
                e
            );
            None
        }
    }
}

pub fn new_socket_addr(port: u16) -> SocketAddr {
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
}
