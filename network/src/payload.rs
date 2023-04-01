use crate::*;
use serde::Serialize;

pub const PAYLOAD_LEN: usize = 509;

#[derive(Clone, Serialize, Debug)]
pub enum Payload {
    ControlPayload(ControlPayload),
    RelayPayload(RelayPayload),
}

impl From<RelayPayload> for Payload {
    fn from(value: RelayPayload) -> Self {
        Self::new_relay_payload(value)
    }
}

impl From<ControlPayload> for Payload {
    fn from(value: ControlPayload) -> Self {
        Self::new_control_payload(value)
    }
}

impl Payload {
    pub fn new_relay_payload(relay_payload: RelayPayload) -> Self {
        Self::RelayPayload(relay_payload)
    }

    pub fn new_control_payload(control_payload: ControlPayload) -> Self {
        Self::ControlPayload(control_payload)
    }

    pub fn is_relay_payload(&self) -> bool {
        if let Self::RelayPayload(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_control_payload(&self) -> bool {
        if let Self::ControlPayload(_) = self {
            return true;
        }
        return false;
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Self::ControlPayload(control_payload) => bincode::serialize(control_payload)
                .expect("[FAILED] Rpc::send_msg --> Unable to serialize message"),
            Self::RelayPayload(relay_payload) => bincode::serialize(relay_payload)
                .expect("[FAILED] Rpc::send_msg --> Unable to serialize message"),
        }
    }

    pub fn into_create(&self) -> Option<CreatePayload> {
        if let Self::ControlPayload(control_payload) = self {
            Some(control_payload.into_create())
        } else {
            None
        }
    }

    pub fn into_created(&self) -> Option<CreatedPayload> {
        if let Self::ControlPayload(control_payload) = self {
            Some(control_payload.into_created())
        } else {
            None
        }
    }

    pub fn into_extend(&self) -> Option<ExtendPayload> {
        if let Self::RelayPayload(relay_payload) = self {
            Some(relay_payload.into_extend())
        } else {
            None
        }
    }

    pub fn into_extended(&self) -> Option<ExtendedPayload> {
        if let Self::RelayPayload(relay_payload) = self {
            Some(relay_payload.into_extended())
        } else {
            None
        }
    }

    // pub fn deserialize_relay_payload(buffer: &[u8]) -> RelayPayload {
    //     bincode::deserialize(&buffer.to_vec()).unwrap()
    // }

    // pub fn deserialize_control_payload(buffer: &[u8]) -> ControlPayload {
    //     bincode::deserialize(&buffer.to_vec()).unwrap()
    // }
}
