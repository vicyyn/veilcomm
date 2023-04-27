use network::Node;

#[derive(Debug, Clone)]
pub enum PendingResponse {
    Pong,
    Extended(Node),
    Created(Option<Node>), // optional node is the node trying to extend the circuit
    Connected(u16),
    IntroEstablished(Node),
    RendPointEstablished(Node),
    IntroduceAck(Node),
}
