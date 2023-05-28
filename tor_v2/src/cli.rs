use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value_t = 8001)]
    pub port: u16,
}
