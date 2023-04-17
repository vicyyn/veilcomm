use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::PendingResponse;
use network::Node;

pub struct PendingResponses(Arc<RwLock<HashMap<Node, PendingResponse>>>);

impl PendingResponses {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, node: Node) -> Option<PendingResponse> {
        match self.0.read().unwrap().get(&node) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn insert(&self, node: Node, pending_response: PendingResponse) {
        self.0.write().unwrap().insert(node, pending_response);
    }

    pub fn pop(&self, node: Node) {
        match self.0.write().unwrap().remove(&node) {
            Some(value) => {
                println!(
                    "[SUCCESS] PendingResponses::pop --> Received Valid Response --  {:?}",
                    value
                )
            }
            None => {
                println!("[WARNING] PendingResponses::pop --> Received Non Expected Response -- ")
            }
        }
    }
}
