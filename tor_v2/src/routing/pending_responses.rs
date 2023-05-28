use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::PendingResponse;

#[derive(Debug)]
pub struct PendingResponses(Arc<RwLock<HashMap<u16, PendingResponse>>>);

impl PendingResponses {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, circ_id: u16) -> Option<PendingResponse> {
        match self.0.read().unwrap().get(&circ_id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn insert(&self, circ_id: u16, pending_response: PendingResponse) {
        self.0.write().unwrap().insert(circ_id, pending_response);
    }

    pub fn pop(&self, circ_id: u16) -> Option<PendingResponse> {
        self.0.write().unwrap().remove(&circ_id)
    }
}
