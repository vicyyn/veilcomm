// Communicate with other peers
use crate::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread;

pub struct Connection {
    pub writer: SyncSender<Cell>,
}

impl Connection {
    pub fn new(stream: TcpStream, events_sender: Sender<ConnectionEvent>) -> Connection {
        Connection::open_read(stream.try_clone().unwrap(), events_sender);
        Self {
            writer: Connection::open_write(stream.try_clone().unwrap()),
        }
    }

    pub fn write(&self, cell: Cell) {
        self.writer.send(cell).unwrap();
    }

    fn open_write(stream: TcpStream) -> SyncSender<Cell> {
        let mut stream = stream.try_clone().unwrap();
        let (write_sender, write_receiver): (SyncSender<Cell>, Receiver<Cell>) =
            mpsc::sync_channel(0);
        thread::spawn(move || loop {
            println!("{:?}", write_receiver);
            let cell = write_receiver
                .recv()
                .expect("[FAILED] Connection::open_write --> Error reading from socket");
            println!("Sending {:?} to {:?}", cell, stream.peer_addr());
            stream.write(&cell.serialize()).unwrap();
        });
        return write_sender;
    }

    fn open_read(stream: TcpStream, events_sender: Sender<ConnectionEvent>) {
        let mut buffer = [0u8; CELL_SIZE];
        let mut stream = stream.try_clone().unwrap();
        let node: Node = stream.peer_addr().unwrap().into();
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

                    events_sender
                        .send(ConnectionEvent::ReceiveCell(
                            node,
                            Cell::deserialize(&buffer),
                        ))
                        .unwrap();
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
