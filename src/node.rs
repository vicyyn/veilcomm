use crate::*;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Node {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub id: Id,
}

impl From<SocketAddr> for Node {
    fn from(addr: SocketAddr) -> Self {
        Node::new(
            Ipv4Addr::from_str(&addr.ip().to_string()).unwrap(),
            addr.port(),
        )
    }
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        let addr = format!("{}:{}", ip, port);
        let id = Id::new(addr);
        Node { ip, port, id }
    }

    pub fn get_info(&self) -> String {
        let mut parsed_id = hex::encode(self.id.0);
        parsed_id = parsed_id.to_ascii_uppercase();
        format!("{}:{}:{}", self.ip, self.port, parsed_id)
    }

    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
