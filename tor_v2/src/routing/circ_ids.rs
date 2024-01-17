use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct CircIds(Arc<RwLock<HashMap<u16, u16>>>);

impl CircIds {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, circ_id: u16) -> Option<u16> {
        match self.0.read().unwrap().get(&circ_id) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    pub fn insert(&self, circ_id: u16, circ_id_2: u16) {
        self.0.write_all().unwrap().insert(circ_id, circ_id_2);
        self.0.write_all().unwrap().insert(circ_id_2, circ_id);
    }
}
