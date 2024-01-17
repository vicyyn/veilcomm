use crate::*;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread;

#[derive(Clone)]
pub struct Connection {
    pub writer: SyncSender<Cell>,
}

impl Connection {
    pub fn new(stream: TcpStream, events_sender: Sender<TorEvent>) -> Connection {
        Connection::open_read(stream.try_clone().unwrap(), events_sender);
        Self {
            writer: Connection::open_write(stream.try_clone().unwrap()),
        }
    }

    pub fn write(&self, cell: Cell) {
        self.write_allr.send(cell).unwrap();
    }

    fn open_write(stream: TcpStream) -> SyncSender<Cell> {
        let mut stream = stream.try_clone().unwrap();
        let (write_sender, write_receiver): (SyncSender<Cell>, Receiver<Cell>) =
            mpsc::sync_channel(0);
        thread::spawn(move || loop {
            let cell = write_receiver
                .recv()
                .expect("[FAILED] Connection::open_write --> Error reading from socket");
            stream.write_all(&cell.serialize()).unwrap();
            println!(
                "[SUCCESS] Connection::open_write --> Sent cell ({:?}) to {:?}",
                cell.command,
                stream.peer_addr().unwrap()
            );
        });
        return write_sender;
    }

    fn open_read(stream: TcpStream, events_sender: Sender<TorEvent>) {
        let mut buffer = [0u8; CELL_SIZE];
        let mut stream = stream.try_clone().unwrap();
        if let SocketAddr::V4(socket_address) = stream.peer_addr().unwrap() {
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
                            "[SUCCESS] Connection::receive --> Received : {} bytes from {:?}",
                            n,
                            stream.peer_addr().unwrap()
                        );

                        events_sender
                            .send(TorEvent::ReceiveCell(
                                socket_address,
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
}
