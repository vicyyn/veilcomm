use crate::*;

#[derive(Debug)]
pub enum Event {
    NewConnection(Node, TcpStream),
    ReceiveCell(Node, Cell),
}

impl Event {
    pub fn initialize_channels() -> (Sender<Event>, Receiver<Event>) {
        mpsc::channel()
    }
}
