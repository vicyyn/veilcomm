use crate::*;

pub struct Circuit {
    pub predecessor: CircuitNode,
    pub successor: Option<CircuitNode>,
}

impl Circuit {
    pub fn new(predecessor: CircuitNode, successor: Option<CircuitNode>) -> Self {
        Self {
            predecessor,
            successor,
        }
    }

    pub fn set_successor(&mut self, successor: Option<CircuitNode>) {
        self.successor = successor;
    }
}
