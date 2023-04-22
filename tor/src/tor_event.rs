use crate::{Circuits, Connections, Keys, PendingResponses, Streams, TorState, *};
use directory::{Relay, Relays, UserDescriptor};
use network::{Cell, ConnectionEvent, Node};
use std::{
    future::Future,
    io::Result,
    net::TcpStream,
    sync::{mpsc::Sender, Arc, RwLock},
};

#[derive(Debug)]
pub enum TorEvent {
    Connect(Node),
    NewConnection(Node, TcpStream),
    SendCell(Node, Cell),
    SendExtend(Node, Node),
    SendCreate(Node),
    OpenStream(Node, Node),
    EstablishIntro(Node),
    GetRelays,
    GetUserDescriptors,
    SendRelay(Relay),
    SendUserDescriptor(UserDescriptor),
}

pub fn process_tor_event(tor_event: TorEvent, tor_state: TorState) -> Result<(), Error> {
    std::thread::spawn(move || match tor_event {
        TorEvent::Connect(node) => {
            println!("[INFO] tor::process_connection_event --> Connect event");
            connect_to_peer(node, connection_events_sender.clone());
        }
        TorEvent::EstablishIntro(node) => {}
        TorEvent::NewConnection(node, stream) => {}
        TorEvent::Connect(_) => todo!(),
        TorEvent::SendCell(_, _) => todo!(),
        TorEvent::SendExtend(_, _) => todo!(),
        TorEvent::SendCreate(_) => todo!(),
        TorEvent::OpenStream(_, _) => todo!(),
        TorEvent::GetRelays => todo!(),
        TorEvent::GetUserDescriptors => todo!(),
        TorEvent::SendRelay(_) => todo!(),
        TorEvent::SendUserDescriptor(_) => todo!(),
    });
}
