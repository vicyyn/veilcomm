use std::net::SocketAddrV4;

#[derive(Debug, Clone)]
pub enum PendingResponse {
    Pong,
    Extended(SocketAddrV4),
    Created(Option<SocketAddrV4>), // optional node is the node trying to extend the circuit
    Connected(u16),
    IntroEstablished(SocketAddrV4, [u8; 32]),
    RendPointEstablished(SocketAddrV4),
    IntroduceAck(SocketAddrV4),
    Rendezvous2([u8; 32]),
}
