use network::Node;

#[derive(Debug)]
pub enum PendingResponse {
    Pong,
    Extended,
    Created(Option<Node>), // optional node is the node trying to extend the circuit
}
