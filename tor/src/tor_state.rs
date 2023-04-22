use crate::{Circuits, Connections, Keys, PendingResponses, Relays, Streams};

pub struct TorState {
    pub connections: Connections,
    pub pending_responses: PendingResponses,
    pub keys: Keys,
    pub circuits: Circuits,
    pub relays: Relays,
    pub streams: Streams,
}

impl TorState {
    pub fn new() -> Self {
        Self {
            connections: Connections::new(),
            pending_responses: PendingResponses::new(),
            keys: Keys::new(),
            circuits: Circuits::new(),
            relays: Relays::new(),
            streams: Streams::new(),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            pending_responses: self.pending_responses.clone(),
            keys: self.keys.clone(),
            circuits: self.circuits.clone(),
            relays: self.relays.clone(),
            streams: self.streams.clone(),
        }
    }
}
