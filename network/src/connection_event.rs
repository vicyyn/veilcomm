use crate::*;
use std::net::TcpStream;

#[derive(Debug)]
pub enum ConnectionEvent {
    Connect(Node),
    NewConnection(Node, TcpStream),
    SendCell(Node, Cell),
    SendExtend(Node, Node),
    SendCreate(Node),
    ReceiveCell(Node, Cell),
    OpenStream(Node, Node),
    EstablishIntro(Node),
    PublishUserDescriptor,
    FetchFromDirectory,
}
