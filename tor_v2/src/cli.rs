use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value_t = 8001)]
    pub port: u16,

    #[arg(value_enum)]
    pub role: Role,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Role {
    Directory,
    Client,
    Relay,
}
