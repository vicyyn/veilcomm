use crate::{Circuits, Connections, Keys, PendingResponses, Streams, TorState, *};
use directory::{Relay, Relays, UserDescriptor};
use network::{Cell, Node};
use std::{
    future::Future,
    io::Result,
    net::TcpStream,
    sync::{mpsc::Sender, Arc, RwLock},
};

#[derive(Debug)]
pub enum TorEvent {
    ConnectToPeer(Node),
    ConnectToDirectory(Node),
    NewPeerConnection(Node, TcpStream),

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

pub fn process_tor_event(
    tor_event: TorEvent,
    tor_state: TorState,
    tor_event_sender: Sender<TorEvent>,
) -> Result<(), Error> {
    std::thread::spawn(move || match tor_event {
        TorEvent::ConnectToPeer(node) => {
            println!("[INFO] tor::process_connection_event --> Connect to peer event");
            let stream = connect(node)?;
            tor_state
                .connections
                .insert(node, Connection::new(stream, ConnectionType::Peer));
            Ok(())
        }
        TorEvent::ConnectToDirectory(node) => {
            println!("[INFO] tor::process_connection_event --> Connect to directory event");
            let stream = connect(node)?;
            tor_state
                .connections
                .insert(node, Connection::new(stream, ConnectionType::Directory));
            Ok(())
        }
        TorEvent::NewPeerConnection(node, stream) => {
            tor_state
                .connections
                .insert(node, Connection::new(stream, ConnectionType::Peer));
            Ok(())
        }
        TorEvent::EstablishIntro(node) => {}
        TorEvent::SendCell(node, cell) => {
            let connection = tor_state.connections.get(&node).unwrap();
            connection.send_cell(cell);
            Ok(())
        }
        TorEvent::SendExtend(_, _) => todo!(),
        TorEvent::SendCreate(_) => todo!(),
        TorEvent::OpenStream(_, _) => todo!(),
        TorEvent::GetRelays => todo!(),
        TorEvent::GetUserDescriptors => todo!(),
        TorEvent::SendRelay(_) => todo!(),
        TorEvent::SendUserDescriptor(_) => todo!(),
    });
}
