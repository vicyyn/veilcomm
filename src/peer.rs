use crate::*;
use std::net::TcpStream;

pub struct Peer {
    pub node: Node,
    pub connection_channels: ConnectionChannels,
}

impl Peer {
    pub fn new(stream: TcpStream) -> Self {
        let node = stream.peer_addr().unwrap().into();
        let connection_channels = Connection::open(stream);

        Self {
            node,
            connection_channels,
        }
    }

    pub fn send_cell(&self, cell: Cell) {
        self.connection_channels.write_sender.send(cell).unwrap();
    }
}
