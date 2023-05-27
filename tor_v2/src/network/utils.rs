use crate::Connections;
pub use log::{error, info};
use std::{
    net::{SocketAddrV4, TcpListener, TcpStream},
    thread,
};

pub fn connect_to_directory(socket_address: SocketAddrV4, connections: Connections) {
    match TcpStream::connect(socket_address) {
        Ok(tcp_stream) => {
            info!("Connected to Socket: {:?}", socket_address);
            connections.insert(socket_address, tcp_stream);
        }
        Err(e) => {
            error!("Error Connecting to Socket: {}", e);
        }
    }
}

pub fn listen_for_connections(socket_address: SocketAddrV4, connections: Connections) {
    thread::spawn(move || loop {
        let socket = TcpListener::bind(socket_address).unwrap();

        match socket.accept() {
            Ok((tcp_stream, addr)) => {
                info!("New client connected: {:?}", addr);
                connections.insert(socket_address, tcp_stream);
            }
            Err(e) => {
                error!("Error accepting client connection: {}", e);
            }
        }
    });
}
