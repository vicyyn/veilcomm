use crate::*;
use std::net::Ipv4Addr;

// Define the Directory struct
pub struct Directory {
    bootstrap_nodes: Vec<Node>,
}

impl Directory {
    pub fn new() -> Self {
        Self {
            bootstrap_nodes: vec![
                Node::new(Ipv4Addr::new(127, 0, 0, 1), 8000),
                Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001),
                Node::new(Ipv4Addr::new(127, 0, 0, 1), 8002),
                Node::new(Ipv4Addr::new(127, 0, 0, 1), 8003),
                Node::new(Ipv4Addr::new(127, 0, 0, 1), 8004),
                Node::new(Ipv4Addr::new(127, 0, 0, 1), 8005),
            ],
        }
    }

    pub fn get_bootstrap_nodes(&self) -> Vec<Node> {
        self.bootstrap_nodes.clone()
    }
}
