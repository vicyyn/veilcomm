use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ./program server|client");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "server" => run_server(),
        "client" => run_client(),
        _ => {
            eprintln!("Invalid argument. Usage: ./program server|client");
            std::process::exit(1);
        }
    }
}

fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 512];
                stream.read(&mut buffer)?;

                let message = String::from_utf8_lossy(&buffer[..]);
                println!("Received message: {}", message);

                stream.write_all(b"Message received.")?;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    Ok(())
}

fn run_client() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    let message = "Hello, server!";
    stream.write_all(message.as_bytes())?;

    let mut buffer = [0; 512];
    stream.read(&mut buffer)?;

    let message = String::from_utf8_lossy(&buffer[..]);
    println!("Received message: {}", message);

    Ok(())
}
