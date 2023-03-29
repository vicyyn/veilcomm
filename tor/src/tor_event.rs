use network::{Cell, Node};

#[derive(Debug)]
pub enum TorEvent {
    Connect(Node),
    SendExtend(Node, Node),
}
