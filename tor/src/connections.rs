use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{Connection, Node};

pub struct Connections(Arc<RwLock<HashMap<Node, Connection>>>);

impl Default for Connections {
    fn default() -> Self {
        Self::new()
    }
}

impl Connections {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, node: Node) -> Option<Connection> {
        self.0.read().unwrap().get(&node).cloned()
    }

    pub fn insert(&self, node: Node, connection: Connection) {
        self.0.write().unwrap().insert(node, connection);
    }
}
