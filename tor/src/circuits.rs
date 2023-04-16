use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{Circuit, CircuitNode};

pub struct Circuits(Arc<RwLock<HashMap<u16, Circuit>>>);

impl Circuits {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, circ_id: u16) -> Circuit {
        self.0.read().unwrap().get(&circ_id).unwrap().clone()
    }

    pub fn add_successor(&self, circ_id: u16, circuit_node: CircuitNode) {
        self.0
            .write()
            .unwrap()
            .get_mut(&circ_id)
            .unwrap()
            .add_successor(circuit_node);
    }

    pub fn set_successor(&self, circ_id: u16, circuit_node: Option<CircuitNode>) {
        self.0
            .write()
            .unwrap()
            .get_mut(&circ_id)
            .unwrap()
            .set_successor(circuit_node);
    }

    pub fn add_circuit(&self, circ_id: u16, circuit: Circuit) {
        self.0.write().unwrap().insert(circ_id, circuit);
    }
}
