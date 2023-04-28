use crate::*;
use std::net::TcpStream;

#[derive(Debug)]
pub enum ConnectionEvent {
    Connect(Node),
    NewConnection(Node, TcpStream),
    ReceiveCell(Node, Cell),

    SendCell(Cell),
    SendExtend(u16, Node),
    SendCreate(u16, Node),
    OpenStream(u16, Node),
    EstablishIntro(u16),
    EstablishRendPoint(u16),
    Introduce1(u16),

    PublishUserDescriptor,
    FetchFromDirectory,
}
