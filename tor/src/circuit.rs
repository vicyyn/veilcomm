use crate::*;

#[derive(Debug, Clone)]
pub enum Circuit {
    OpCircuit(OpCircuit),
    OrCircuit(OrCircuit),
}

#[derive(Debug, Clone)]
pub struct OpCircuit {
    successors: Vec<CircuitNode>,
}

#[derive(Debug, Clone)]
pub struct OrCircuit {
    predecessor: CircuitNode,
    successor: Option<CircuitNode>,
}

impl OpCircuit {
    pub fn new() -> Self {
        Self { successors: vec![] }
    }

    pub fn add_successor(&mut self, successor: CircuitNode) {
        self.successors.push(successor);
    }

    pub fn get_successors(&self) -> Vec<CircuitNode> {
        self.successors.clone()
    }

    pub fn get_first(&self) -> CircuitNode {
        self.successors[0].clone()
    }

    pub fn encrypt_cell(&self, cell: Cell) -> Cell {
        let mut new_cell = cell.clone();
        for circuit_node in self.get_successors().iter().rev() {
            new_cell.payload = circuit_node.encrypt_payload(new_cell.payload.clone());
        }
        return new_cell;
    }

    pub fn decrypt_cell(&self, cell: Cell) -> Cell {
        let mut new_cell = cell.clone();
        for circuit_node in self.get_successors() {
            new_cell.payload = circuit_node.decrypt_payload(new_cell.payload.clone());
        }
        return new_cell;
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

    pub fn decrypt_cell(&self, cell: Cell) -> Cell {
        let mut new_cell = cell.clone();
        new_cell.payload = self.predecessor.decrypt_payload(cell.payload.clone());
        return new_cell;
    }

    pub fn encrypt_cell(&self, cell: Cell) -> Cell {
        let mut new_cell = cell.clone();
        new_cell.payload = self
            .predecessor
            .clone()
            .encrypt_payload(cell.payload.clone());
        return new_cell;
    }

    pub fn is_forward(&self, source: Node) -> bool {
        if self.predecessor.node.eq(&source) {
            return true;
        }
        return false;
    }

    pub fn is_backward(&self, source: Node) -> bool {
        return !self.is_forward(source);
    }

    pub fn handle_cell(&self, source: Node, cell: Cell) -> Cell {
        if self.is_forward(source) {
            self.decrypt_cell(cell)
        } else {
            self.encrypt_cell(cell)
        }
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
            return Some(op_circuit.get_successors());
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

    pub fn handle_cell(&self, source: Node, cell: Cell) -> Cell {
        match &self {
            Self::OrCircuit(or_circuit) => or_circuit.handle_cell(source, cell),
            Self::OpCircuit(op_circuit) => op_circuit.decrypt_cell(cell),
        }
    }

    pub fn get_cell_destination(&self, source: Node) -> Option<CircuitNode> {
        if let Self::OrCircuit(or_circuit) = self {
            if or_circuit.is_forward(source) {
                return or_circuit.successor.clone();
            } else {
                return Some(or_circuit.predecessor.clone());
            }
        }
        None
    }
}
