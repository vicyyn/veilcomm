use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::Node;

#[derive(Debug, Clone, Default)]
pub struct Streams(Arc<RwLock<HashMap<u16, Node>>>);

impl Streams {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn get(&self, id: u16) -> Option<Node> {
        if let Some(node) = self.0.read().unwrap().get(&id) {
            return Some(*node);
        }
        None
    }

    pub fn insert(&self, id: u16, node: Node) {
        self.0.write().unwrap().insert(id, node);
    }
}
