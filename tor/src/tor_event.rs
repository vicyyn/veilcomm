use std::sync::{mpsc::Sender, Arc, RwLock};

use directory::{Relay, Relays, UserDescriptor};
use network::{Cell, ConnectionEvent, Node};

use crate::{Circuits, Connections, Keys, PendingResponses, Streams};

#[derive(Debug)]
pub enum TorEvent {
    Connect(Node),
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

fn process_tor_event(
    tor_event: TorEvent,
    connections: Connections,
    pending_responses: PendingResponses,
    network_events_sender: Sender<ConnectionEvent>,
    keys: Arc<RwLock<Keys>>,
    circuits: Circuits,
    relays: Arc<Relays>,
    streams: Streams,
) {
    std::thread::spawn(move || match tor_event {
        TorEvent::Connect(node) => {

        }
        TorEvent::EstablishIntro(node) => {}
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
