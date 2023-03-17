use crate::*;
use std::net::TcpStream;

pub struct Peer {
    pub node: Node,
    pub connection_channels: ConnectionChannels,
}

impl Peer {
    pub fn new(node: Node, stream: TcpStream) -> Self {
        let connection_channels = Connection::open(stream);

        Self {
            node,
            connection_channels,
        }
    }
}
