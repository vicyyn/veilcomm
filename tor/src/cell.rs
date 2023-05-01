// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.
use crate::*;

pub const CELL_SIZE: usize = 512;

#[derive(Clone, Debug)]
pub struct Cell {
    pub circ_id: u16,
    pub command: u8,
    pub payload: Payload,
}

impl Cell {
    pub fn new(circ_id: u16, command: u8, payload: Payload) -> Self {
        Self {
            circ_id,
            command,
            payload,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.extend(self.circ_id.to_le_bytes());
        serialized.push(self.command);
        serialized.extend(self.payload.serialize());
        return serialized;
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        let circ_id = u16::from_le_bytes(buffer[0..2].try_into().unwrap());
        let command = buffer[2];
        let payload: Payload = if CellCommand::Relay as u8 == command {
            RelayPayload::deserialize(&buffer[3..]).into()
        } else {
            ControlPayload::deserialize(&buffer[3..]).into()
        };
        return Cell::new(circ_id, command, payload);
    }

    pub fn new_create_cell(circ_id: u16, control_payload: ControlPayload) -> Self {
        Self::new(circ_id, 1, control_payload.into())
    }

    pub fn new_created_cell(circ_id: u16, control_payload: ControlPayload) -> Self {
        Self::new(circ_id, 2, control_payload.into())
    }

    pub fn new_relay_cell(circ_id: u16, relay_payload: RelayPayload) -> Self {
        Self::new(circ_id, 3, relay_payload.into())
    }
}
