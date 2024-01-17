use crate::*;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct _Relays(Vec<Relay>);

impl Default for _Relays {
    fn default() -> Self {
        Self::new()
    }
}

impl _Relays {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_relay(&self, address: SocketAddr) -> Option<Relay> {
        self.0.clone().into_iter().find(|x| x.address.eq(&address))
    }

    pub fn add_relay(&mut self, relay: Relay) {
        self.0.push(relay);
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("[FAILED] Relays::serialize --> Unable to serialize")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer)
            .expect("[FAILED] Relays::deserialize --> Unable to deserialize")
    }

    pub fn set(&mut self, relays: Vec<Relay>) {
        self.0.clear();
        for relay in relays {
            self.0.push(relay);
        }
    }
}

pub struct Relays(Arc<RwLock<_Relays>>);

impl Default for Relays {
    fn default() -> Self {
        Self::new()
    }
}

impl Relays {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(_Relays::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get_relay(&self, address: SocketAddr) -> Option<Relay> {
        self.0.read().unwrap().get_relay(address)
    }

    pub fn add_relay(&self, relay: Relay) {
        self.0.write().unwrap().add_relay(relay);
    }

    pub fn get_relays(&self) -> Vec<Relay> {
        self.0.read().unwrap().0.clone()
    }

    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.0.read().unwrap().serialize()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        Self(Arc::new(RwLock::new(_Relays::deserialize(buffer))))
    }

    pub fn set(&self, relays: Self) {
        self.0.write().unwrap().set(relays.get_relays());
    }
}
