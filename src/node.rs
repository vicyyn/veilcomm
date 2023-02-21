use crate::*;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Node {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub key: Key,
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        let addr = format!("{}:{}", ip, port);
        let key = Key::new(addr);
        Node { ip, port, key }
    }

    pub fn get_info(&self) -> String {
        let mut parsed_id = hex::encode(self.key.0);
        parsed_id = parsed_id.to_ascii_uppercase();
        format!("{}:{}:{}", self.ip, self.port, parsed_id)
    }

    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
