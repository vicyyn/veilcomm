// UDP connection with another node in the network.
use crate::*;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::{Binary, Debug, Error, Formatter};
use std::net::UdpSocket;
use std::sync::{mpsc, Arc, Mutex};

const CELL_PAYLOAD_SIZE: usize = 509;

pub struct Connection {
    pub socket: Arc<UdpSocket>,
    pub pending: Arc<Mutex<HashMap<Key, mpsc::Sender<Option<Response>>>>>,
    pub node: Node,
}
