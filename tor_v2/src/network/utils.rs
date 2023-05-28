use crate::TorEvent;
use std::{
    net::{SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    sync::mpsc::Sender,
    thread,
};

pub fn listen_for_connections(socket_address: SocketAddrV4, sender: Sender<TorEvent>) {
    thread::spawn(move || loop {
        let socket = TcpListener::bind(socket_address)
            .expect("[FAILED] tor::listen_for_connections --> Error while binding TcpSocket to specified addr");

        match socket.accept() {
            Ok((stream, addr)) => {
                println!(
                    "[SUCCESS] tor::listen_for_connections - New client connected: {:?}",
                    addr
                );
                if let SocketAddr::V4(socket_address) = addr {
                    sender
                        .send(TorEvent::NewConnection(socket_address, stream))
                        .unwrap()
                }
            }
            Err(e) => {
                println!(
                    "[FAILED] tor::listen_for_connections - Error accepting client connection: {}",
                    e
                );
            }
        }
    });
}

pub fn connect_to_peer(socket_address: SocketAddrV4, sender: Sender<TorEvent>) {
    match TcpStream::connect(socket_address) {
        Ok(stream) => {
            println!(
                "[SUCCESS] tor::connect_to_peer --> Connected to Peer: {:?}",
                socket_address
            );
            sender
                .send(TorEvent::NewConnection(
                    socket_address,
                    stream.try_clone().unwrap(),
                ))
                .unwrap();
        }
        Err(e) => {
            println!(
                "[FAILED] tor::connect_to_peer --> Error Connecting to Peer: {}",
                e
            );
        }
    }
}
