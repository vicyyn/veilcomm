use crate::*;

#[derive(Clone)]
pub enum Circuit {
    OpCircuit(OpCircuit),
    OrCircuit(OrCircuit),
}

#[derive(Debug, Clone)]
pub struct OpCircuit {
    pub successors: Vec<CircuitNode>,
}

#[derive(Debug, Clone)]
pub struct OrCircuit {
    pub predecessor: CircuitNode,
    pub successor: Option<CircuitNode>,
}

impl OpCircuit {
    pub fn new() -> Self {
        Self { successors: vec![] }
    }

    pub fn add_successor(&mut self, successor: CircuitNode) {
        self.successors.push(successor);
    }
}

impl OrCircuit {
    pub fn new(predecessor: CircuitNode, successor: Option<CircuitNode>) -> Self {
        Self {
            predecessor,
            successor,
        }
    }

    pub fn get_successor(&self) -> Option<CircuitNode> {
        return self.successor.clone();
    }

    pub fn get_predecessor(&self) -> CircuitNode {
        return self.predecessor.clone();
    }

    pub fn set_successor(&mut self, successor: Option<CircuitNode>) {
        self.successor = successor;
    }
}

impl Circuit {
    pub fn new_op_circuit() -> Self {
        Self::OpCircuit(OpCircuit::new())
    }

    pub fn new_or_circuit(predecessor: CircuitNode, successor: Option<CircuitNode>) -> Self {
        Self::OrCircuit(OrCircuit::new(predecessor, successor))
    }

    pub fn is_op_circuit(&self) -> bool {
        if let Self::OpCircuit(_) = self {
            return true;
        }
        return false;
    }

    pub fn is_or_circuit(&self) -> bool {
        if let Self::OrCircuit(_) = self {
            return true;
        }
        return false;
    }

    pub fn add_successor(&mut self, successor: CircuitNode) {
        if let Self::OpCircuit(op_circuit) = self {
            op_circuit.add_successor(successor);
        }
    }

    pub fn set_successor(&mut self, successor: Option<CircuitNode>) {
        if let Self::OrCircuit(or_circuit) = self {
            or_circuit.set_successor(successor);
        }
    }

    pub fn get_successors(&self) -> Option<Vec<CircuitNode>> {
        if let Self::OpCircuit(op_circuit) = self {
            return Some(op_circuit.successors.clone());
        }
        return None;
    }

    pub fn get_successor(&self) -> Option<CircuitNode> {
        if let Self::OrCircuit(or_circuit) = self {
            return or_circuit.get_successor();
        }
        return None;
    }

    pub fn get_predecessor(&self) -> Option<CircuitNode> {
        if let Self::OrCircuit(or_circuit) = self {
            return Some(or_circuit.get_predecessor());
        }
        return None;
    }
}
