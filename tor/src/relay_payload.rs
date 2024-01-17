use crate::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelayPayload {
    pub command: u8,
    pub recognized: u16,
    pub stream_id: u16,
    pub digest: u32,
    pub length: u16,
    #[serde(with = "BigArray")]
    pub data: [u8; PAYLOAD_LEN - 11],
}

impl From<Payload> for RelayPayload {
    fn from(value: Payload) -> Self {
        bincode::deserialize(&value.serialize().to_vec()).unwrap()
    }
}

impl RelayPayload {
    pub fn new(data: [u8; PAYLOAD_LEN]) -> Self {
        Self {
            command: data[0],
            recognized: u16::from_le_bytes(data[1..3].try_into().unwrap()),
            stream_id: u16::from_le_bytes(data[3..5].try_into().unwrap()),
            digest: u32::from_le_bytes(data[5..9].try_into().unwrap()),
            length: u16::from_le_bytes(data[9..11].try_into().unwrap()),
            data: data[11..].try_into().unwrap(),
        }
    }

    pub fn into_extend(&self) -> ExtendPayload {
        ExtendPayload {
            address: self.data[..4].try_into().unwrap(),
            port: u16::from_le_bytes(self.data[4..6].try_into().unwrap()),
            onion_skin: OnionSkin::deserialize(&self.data[6..(6 + ONION_SKIN_LEN)]),
        }
    }

    pub fn into_extended(&self) -> ExtendedPayload {
        let mut dh_key = [0; 256];
        dh_key.copy_from_slice(&self.data[0..256]);
        ExtendedPayload { dh_key }
    }

    pub fn into_establish_intro(&self) -> EstablishIntroPayload {
        let mut address = [0; 32];
        address.copy_from_slice(&self.data[0..32]);
        EstablishIntroPayload { address }
    }

    pub fn into_introduce1(&self) -> Introduce1Payload {
        let mut address = [0; 32];
        address.copy_from_slice(&self.data[0..32]);
        let mut ip = [0; 4];
        ip.copy_from_slice(&self.data[32..36]);
        let port = u16::from_le_bytes(self.data[36..38].try_into().unwrap());
        let mut cookie = [0; 20];
        cookie.copy_from_slice(&self.data[38..58]);
        let onion_skin = OnionSkin::deserialize(&self.data[58..(58 + ONION_SKIN_LEN)]);
        Introduce1Payload {
            address,
            ip,
            port,
            cookie,
            onion_skin,
        }
    }

    pub fn into_introduce2(&self) -> Introduce2Payload {
        let mut ip = [0; 4];
        ip.copy_from_slice(&self.data[0..4]);
        let port = u16::from_le_bytes(self.data[4..6].try_into().unwrap());
        let mut cookie = [0; 20];
        cookie.copy_from_slice(&self.data[6..26]);
        let onion_skin = OnionSkin::deserialize(&self.data[26..(26 + ONION_SKIN_LEN)]);
        Introduce2Payload {
            ip,
            port,
            cookie,
            onion_skin,
        }
    }

    pub fn into_establish_rend_point(&self) -> EstablishRendPointPayload {
        let mut cookie = [0; 20];
        cookie.copy_from_slice(&self.data[0..20]);
        EstablishRendPointPayload { cookie }
    }

    pub fn into_begin(&self) -> BeginPayload {
        BeginPayload {
            address: self.data[..4].try_into().unwrap(),
            port: u16::from_le_bytes(self.data[4..6].try_into().unwrap()),
        }
    }

    pub fn into_connected(&self) -> ConnectedPayload {
        ConnectedPayload {
            address: self.data[..4].try_into().unwrap(),
            port: u16::from_le_bytes(self.data[4..6].try_into().unwrap()),
        }
    }

    pub fn into_introduce_ack(&self) -> IntroduceAckPayload {
        IntroduceAckPayload {
            status: self.data[0],
        }
    }

    pub fn into_rendezvous1(&self) -> Rendezvous1Payload {
        Rendezvous1Payload {
            cookie: self.data[0..20].try_into().unwrap(),
            dh_key: self.data[20..276].try_into().unwrap(),
        }
    }

    pub fn into_rendezvous2(&self) -> Rendezvous2Payload {
        Rendezvous2Payload {
            dh_key: self.data[0..256].try_into().unwrap(),
        }
    }

    pub fn new_rendezvous2_payload(rendezvous2_payload: Rendezvous2Payload) -> Self {
        let data = rendezvous2_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 37,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_rendezvous1_payload(
        rendezvous1_payload: Rendezvous1Payload,
        stream_id: u16,
    ) -> Self {
        let data = rendezvous1_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 36,
            recognized: 0,
            stream_id,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_extend_payload(extend_payload: ExtendPayload) -> Self {
        let data = extend_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 6,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_extended_payload(extended_payload: ExtendedPayload) -> Self {
        let data = extended_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 7,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_data_payload(data: &[u8], stream_id: u16) -> Self {
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(data);
        Self {
            command: 2,
            recognized: 0,
            stream_id,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_establish_intro_payload(establish_intro_payload: EstablishIntroPayload) -> Self {
        let data = establish_intro_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 32,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_introduce1_payload(introduce1_payload: Introduce1Payload) -> Self {
        let data = introduce1_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 34,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_introduce2_payload(introduce2_payload: Introduce2Payload) -> Self {
        let data = introduce2_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 35,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_intro_established_payload() -> Self {
        Self {
            command: 38,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: [0; PAYLOAD_LEN - 11],
        }
    }

    pub fn new_establish_rend_point_payload(
        establish_rend_point_payload: EstablishRendPointPayload,
    ) -> Self {
        let data = establish_rend_point_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 33,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_rend_point_established_payload() -> Self {
        Self {
            command: 39,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: [0; PAYLOAD_LEN - 11],
        }
    }

    pub fn new_begin_payload(stream_id: u16, begin_payload: BeginPayload) -> Self {
        let data = begin_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 1,
            recognized: 0,
            stream_id,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_connected_payload(stream_id: u16, connected_payload: ConnectedPayload) -> Self {
        let data = connected_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 4,
            recognized: 0,
            stream_id,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn new_introduce_ack_payload(introduce_ack_payload: IntroduceAckPayload) -> Self {
        let data = introduce_ack_payload.serialize();
        let mut buffer = [0; PAYLOAD_LEN - 11];
        buffer[..data.len()].copy_from_slice(&data);
        Self {
            command: 40,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: buffer,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer)
            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
    }
}

impl Default for RelayPayload {
    fn default() -> Self {
        Self {
            command: 0,
            recognized: 0,
            stream_id: 0,
            digest: 0,
            length: 0,
            data: [0; PAYLOAD_LEN - 11],
        }
    }
}
