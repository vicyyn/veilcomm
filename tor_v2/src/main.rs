use clap::Parser;
use std::net::{Ipv4Addr, SocketAddrV4};
use tor_v2::{start_peer, Args};

fn main() {
    env_logger::init();
    let args = Args::parse();
    start_peer(SocketAddrV4::new(Ipv4Addr::LOCALHOST, args.port));
    loop {}
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::thread;

    use tor_v2::{create_circuit, Cell, RelayPayload, TorEvent};

    use super::*;

    #[test]
    fn test_tor() {
        let node1 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8001);
        let node2 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8002);
        let node3 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8003);
        let node4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8004);
        let node5 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8005);
        let node6 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8006);
        let node7 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8007);
        let node8 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8008);

        let node9 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8009);
        let node10 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8010);
        let node11 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8011);

        _ = start_peer(node11);
        _ = start_peer(node10);
        _ = start_peer(node9);
        let (t8, b) = start_peer(node8);
        let _ = start_peer(node7);
        let _ = start_peer(node6);
        let _ = start_peer(node5);
        let _ = start_peer(node4);
        let _ = start_peer(node3);
        let _ = start_peer(node2);
        let (t1, a) = start_peer(node1);

        println!(" First Circuit * * * * * * * * * *");
        create_circuit(0, t1.clone(), node2, node3, node4);

        println!(" Second Circuit * * * * * * * * * *");
        create_circuit(0, t8.clone(), node7, node6, node5);
        println!(" * * * * * * * * * *");

        println!(" - - - - - - -");
        t8.send(TorEvent::EstablishIntro(0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t8.send(TorEvent::PublishUserDescriptor).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        t1.send(TorEvent::EstablishRendPoint(0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(TorEvent::OpenStream(0, node5, 0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(TorEvent::FetchFromDirectory).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(TorEvent::Introduce1(0)).unwrap();
        thread::sleep(time::Duration::from_millis(30000));

        println!(" - - - - - - -");
        let relay_payload = RelayPayload::new_data_payload("Hello!".as_bytes(), 3);
        let cell = Cell::new_relay_cell(0, relay_payload);
        t1.send(TorEvent::SendCell(cell.clone())).unwrap();

        println!(" - - - - - - -");
        thread::sleep(time::Duration::from_millis(5000));
        t8.send(TorEvent::SendCell(cell)).unwrap();

        loop {}
    }
}
