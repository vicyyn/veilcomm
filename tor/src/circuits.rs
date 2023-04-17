use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::Circuit;

#[derive(Debug)]
pub struct Circuits(Arc<RwLock<HashMap<u16, Circuit>>>);

impl Circuits {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, circ_id: u16) -> Option<Circuit> {
        match self.0.read().unwrap().get(&circ_id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn insert(&self, circ_id: u16, circuit: Circuit) {
        self.0.write().unwrap().insert(circ_id, circuit);
    }
}
