use crate::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub struct ConnectionChannels {
    pub write_sender: Sender<Cell>,
    pub read_receiver: Receiver<Cell>,
}

impl ConnectionChannels {
    pub fn new(write_sender: Sender<Cell>, read_receiver: Receiver<Cell>) -> Self {
        Self {
            write_sender,
            read_receiver,
        }
    }
}

pub struct Connection {}

impl Connection {
    pub fn new() -> Connection {
        Self {}
    }

    pub fn open(stream: TcpStream) -> ConnectionChannels {
        let (write_sender, write_receiver): (Sender<Cell>, Receiver<Cell>) = mpsc::channel();
        let (read_sender, read_receiver): (Sender<Cell>, Receiver<Cell>) = mpsc::channel();
        Connection::open_write(stream.try_clone().unwrap(), write_receiver);
        Connection::open_read(stream.try_clone().unwrap(), read_sender);
        return ConnectionChannels::new(write_sender, read_receiver);
    }

    pub fn open_write(stream: TcpStream, receiver: Receiver<Cell>) {
        let mut stream = stream.try_clone().unwrap();
        thread::spawn(move || loop {
            let cell = receiver
                .recv()
                .expect("[FAILED] Connection::open_write --> Error reading from socket: {}");
            stream.write(&cell.serialize()).unwrap();
        });
    }

    pub fn open_read(stream: TcpStream, sender: Sender<Cell>) {
        let mut buffer = [0u8; CELL_SIZE];
        let mut stream = stream.try_clone().unwrap();
        thread::spawn(move || loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!(
                        "[WARNING] Connection::receive --> Connection has disconnected from {}",
                        stream.peer_addr().unwrap()
                    );
                    break;
                }
                Ok(n) => {
                    println!(
                        "[INFO] Connection::receive --> Received : {} bytes from {:?}",
                        n,
                        stream.peer_addr().unwrap()
                    );

                    sender.send(Cell::deserialize(&buffer)).unwrap();
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
    }
}
