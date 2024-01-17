use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{Circuit, CircuitNode};

#[derive(Debug)]
pub struct Circuits(Arc<RwLock<HashMap<u16, Circuit>>>);

impl Default for Circuits {
    fn default() -> Self {
        Self::new()
    }
}

impl Circuits {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn get_unused_circ_id(&self) -> u16 {
        (0..=std::u16::MAX)
            .find(|&x| !self.0.read().unwrap().contains_key(&x))
            .unwrap()
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, circ_id: u16) -> Option<Circuit> {
        self.0.read().unwrap().get(&circ_id).cloned()
    }

    pub fn add_successor(&self, circ_id: u16, successor: CircuitNode) {
        self.0
            .write()
            .unwrap()
            .get_mut(&circ_id)
            .unwrap()
            .add_successor(successor);
    }

    pub fn set_successor(&self, circ_id: u16, successor: Option<CircuitNode>) {
        self.0
            .write()
            .unwrap()
            .get_mut(&circ_id)
            .unwrap()
            .set_successor(successor);
    }

    pub fn insert(&self, circ_id: u16, circuit: Circuit) {
        self.0.write().unwrap().insert(circ_id, circuit);
    }
}
