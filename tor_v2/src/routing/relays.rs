use crate::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct _RelayDescriptors(Vec<RelayDescriptor>);

impl _RelayDescriptors {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn new_from(relays: Vec<RelayDescriptor>) -> Self {
        Self(relays)
    }

    pub fn get_relay(&self, address: SocketAddrV4) -> Option<RelayDescriptor> {
        self.0
            .clone()
            .into_iter()
            .find(|x| x.socket_address.eq(&address))
    }

    pub fn add_relay(&mut self, relay: RelayDescriptor) {
        self.0.push(relay);
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Relays::serialize --> Unable to serialize")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] Relays::deserialize --> Unable to deserialize")
    }

    pub fn set(&mut self, relays: Vec<RelayDescriptor>) {
        self.0.clear();
        for relay in relays {
            self.0.push(relay);
        }
    }
}

pub struct RelayDescriptors(Arc<RwLock<_RelayDescriptors>>);

impl RelayDescriptors {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(_RelayDescriptors::new())))
    }

    pub fn new_from(relays: Vec<RelayDescriptor>) -> Self {
        Self(Arc::new(RwLock::new(_RelayDescriptors::new_from(relays))))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get_relay(&self, address: SocketAddrV4) -> Option<RelayDescriptor> {
        self.0.read().unwrap().get_relay(address)
    }

    pub fn add_relay(&self, relay: RelayDescriptor) {
        self.0.write_all().unwrap().add_relay(relay);
    }

    pub fn get_relays(&self) -> Vec<RelayDescriptor> {
        self.0.read().unwrap().0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.0.read().unwrap().serialize()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        Self(Arc::new(RwLock::new(_RelayDescriptors::deserialize(
            buffer,
        ))))
    }

    pub fn set(&self, relays: Self) {
        self.0.write_all().unwrap().set(relays.get_relays());
    }
}
