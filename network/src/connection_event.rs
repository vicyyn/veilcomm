use crate::*;
use std::net::TcpStream;

#[derive(Debug)]
pub enum ConnectionEvent {
    NewConnection(Node, TcpStream),
    ReceiveCell(Node, Cell),
}
