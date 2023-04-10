use crate::NodeRow;

#[derive(Clone, Debug, PartialEq)]
pub struct NodesTable(pub Vec<NodeRow>);

impl NodesTable {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_node_row(&mut self, node_row: NodeRow) {
        self.0.push(node_row)
    }

    pub fn get_tuples(&self) -> Vec<(String, u16, String)> {
        let mut tuples = vec![];
        for node_row in &self.0 {
            tuples.push(node_row.get_tuple())
        }
        return tuples;
    }
}
