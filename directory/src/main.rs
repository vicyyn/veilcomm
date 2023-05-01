use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};

use directory::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let address = SocketAddr::new(
        std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        args[1].parse().unwrap(),
    );
    start_directory(address);
}
