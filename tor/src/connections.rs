use crate::{Connection, Node};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct Connections(Arc<RwLock<HashMap<Node, Connection>>>);

impl Connections {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self, node: Node) -> Option<Connection> {
        match self.0.read().unwrap().get(&node) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn insert(&self, node: Node, connection: Connection) {
        self.0.write().unwrap().insert(node, connection);
    }
}
