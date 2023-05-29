use crate::*;
use std::net::{SocketAddrV4, TcpStream};

#[derive(Debug)]
pub enum TorEvent {
    Connect(SocketAddrV4),
    NewConnection(SocketAddrV4, TcpStream),
    ReceiveCell(SocketAddrV4, Cell),

    SendCell(Cell),
    SendExtend(u16, SocketAddrV4),
    SendCreate(u16, SocketAddrV4),
    OpenStream(u16, SocketAddrV4, u16),
    EstablishIntro(u16),
    EstablishRendPoint(u16),
    Introduce1(u16),

    PublishUserDescriptor,
    FetchFromDirectory,
    InitializePeer(bool),
}
