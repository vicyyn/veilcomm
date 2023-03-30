use network::Node;

#[derive(Debug)]
pub enum PendingResponse {
    Pong,
    Extended,
    Created(Option<Node>), // return extend request
}
