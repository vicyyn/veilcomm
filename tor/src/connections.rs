use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{Connection, Node};

#[derive(Clone, Default)]
pub struct Connections(Arc<RwLock<HashMap<Node, Connection>>>);

impl Connections {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn get(&self, node: Node) -> Option<Connection> {
        self.0.read().unwrap().get(&node).cloned()
    }

    pub fn insert(&self, node: Node, connection: Connection) {
        self.0.write().unwrap().insert(node, connection);
    }
}
