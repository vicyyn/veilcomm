use network::{Cell, Node};

#[derive(Debug)]
pub enum TorEvent {
    Connect(Node),
    Send(Node, Cell),
}
