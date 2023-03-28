use serde::{Deserialize, Serialize};
use std::convert::From;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Node {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl From<SocketAddr> for Node {
    fn from(addr: SocketAddr) -> Self {
        Self::new(
            Ipv4Addr::from_str(&addr.ip().to_string()).unwrap(),
            addr.port(),
        )
    }
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn get_info(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn default() -> Self {
        Self::new(Ipv4Addr::new(127, 0, 0, 1), 8000)
    }
}
