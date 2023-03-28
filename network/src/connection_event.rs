use crate::*;
use std::net::TcpStream;

#[derive(Debug)]
pub enum Event {
    NewConnection(Node, TcpStream),
    ReceiveCell(Node, Cell),
}
