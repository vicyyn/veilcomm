// Payload that gets sent in the cell
use crate::*;
use serde::{Deserialize, Serialize};

//#[derive(Serialize, Deserialize, Debug)]
pub type Payload = CreatePayload;

//pub enum Payload {
//    CreatePayload(CreatePayload),
//}

//impl Payload {
//    pub fn serialize(&self) -> Vec<u8> {
//        bincode::serialize(self).expect("[FAILED] Rpc::send_msg --> Unable to serialize message")
//    }
//
//    pub fn deserialize(buffer: &[u8]) -> Cell {
//        bincode::deserialize(&buffer.to_vec())
//            .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload")
//    }
//
//    pub fn create_create_payload(create_payload: CreatePayload) -> Payload {
//        Payload::CreatePayload(create_payload)
//    }
//
//    pub fn create_created_payload() {}
//    pub fn create_extend_payload() {}
//}
//
//impl Default for Payload {
//    fn default() -> Self {
//        Payload::CreatePayload(CreatePayload::default())
//    }
//}
