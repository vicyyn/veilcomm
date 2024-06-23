use ::rand::{thread_rng, Rng};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

pub type Connections = Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<OwnedWriteHalf>>>>>;

pub fn generate_random_aes_key() -> [u8; 16] {
    let mut rand = thread_rng();
    let key = rand.gen::<[u8; 16]>();
    key
}
