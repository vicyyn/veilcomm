pub mod cli;
pub mod crypto;
pub mod data;
pub mod network;

pub use cli::*;
pub use crypto::*;
pub use data::*;
pub use network::*;

use clap::Parser;

fn main() {
    env_logger::init();
    let args = Args::parse();
    match args.role {
        Role::Directory => todo!(),
        Role::Client => todo!(),
        Role::Relay => todo!(),
    }
}
