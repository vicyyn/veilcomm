use crate::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub enum ConnectionType {
    Directory,
    Peer,
}

pub struct Connection {
    pub stream: TcpStream,
    pub connection_type: ConnectionType,
}

impl Connection {
    pub fn clone(&self) -> Self {
        Self {
            stream: self.stream.try_clone().unwrap(),
            connection_type: self.connection_type,
        }
    }

    pub fn get_type(&self) -> ConnectionType {
        self.connection_type
    }

    pub fn new(
        stream: TcpStream,
        connection_type: ConnectionType,
    ) -> (Connection, Receiver<Vec<u8>>) {
        let read_receive = Connection::open_read(stream.try_clone().unwrap());
        (
            Self {
                // writer: Connection::open_write(stream.try_clone().unwrap()),
                stream: stream.try_clone().unwrap(),
                connection_type,
            },
            read_receive,
        )
    }

    pub fn write(&self, data: Vec<u8>) {
        self.stream.try_clone().unwrap().write(&data).unwrap();
    }

    fn open_read(stream: TcpStream) -> Receiver<Vec<u8>> {
        let (read_sender, read_receiver) = mpsc::channel();
        let node: Node = stream.peer_addr().unwrap().into();
        thread::spawn(move || loop {
            let mut buffer: Vec<u8> = vec![];
            match stream.read_to_end(&mut buffer) {
                Ok(0) => {
                    println!(
                        "[WARNING] Connection::receive --> Connection has disconnected from {}",
                        stream.peer_addr().unwrap()
                    );
                    break;
                }
                Ok(n) => {
                    println!(
                        "[SUCCESS] Connection::receive --> Received : {} bytes from {:?}",
                        n,
                        stream.peer_addr().unwrap()
                    );

                    read_sender.send(buffer).unwrap();
                }
                Err(e) => {
                    println!(
                        "[FAILED] Connection::receive --> Error reading from socket: {}",
                        e
                    );
                    break;
                }
            }
        });
        return read_receiver;
    }
}
