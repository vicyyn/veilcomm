use crate::OnionSkin;
use ::rand::{thread_rng, Rng};
use log::info;
use openssl::{
    bn::BigNum,
    dh::Dh,
    pkey::Private,
    rsa::{Padding, Rsa},
    symm::{decrypt, Cipher},
};
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

pub fn directory_address() -> SocketAddr {
    SocketAddr::from_str("127.0.0.1:8080").unwrap()
}

pub fn api_address() -> SocketAddr {
    SocketAddr::from_str("127.0.0.1:8081").unwrap()
}

pub type Connections = Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<OwnedWriteHalf>>>>>;
