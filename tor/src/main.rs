use tor::*;

use std::{env, net::Ipv4Addr};

fn main() {
    let args: Vec<String> = env::args().collect();
    let _ = start_peer(Node::new(
        Ipv4Addr::new(127, 0, 0, 1),
        args[1].parse().unwrap(),
    ));
    loop {}
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::thread;

    use super::*;
    use directory::{new_socket_addr, start_directory};

    #[test]
    fn test_tor() {
        let node1 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001);
        let node2 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8002);
        let node3 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8003);
        let node4 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8004);
        let node5 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8005);
        let node6 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8006);
        let node7 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8007);
        let node8 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8008);

        let node9 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8009);
        let node10 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8010);
        let node11 = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8011);

        thread::spawn(|| {
            start_directory(new_socket_addr(8090));
        });

        _ = start_peer(node11);
        _ = start_peer(node10);
        _ = start_peer(node9);

        let t8 = start_peer(node8);
        let _ = start_peer(node7);
        let _ = start_peer(node6);
        let _ = start_peer(node5);
        let _ = start_peer(node4);
        let _ = start_peer(node3);
        let _ = start_peer(node2);
        let t1 = start_peer(node1);

        println!(" First Circuit * * * * * * * * * *");
        create_circuit(0, t1.clone(), node2, node3, node4);

        println!(" Second Circuit * * * * * * * * * *");
        create_circuit(0, t8.clone(), node7, node6, node5);
        println!(" * * * * * * * * * *");

        println!(" - - - - - - -");
        t8.send(ConnectionEvent::EstablishIntro(0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t8.send(ConnectionEvent::PublishUserDescriptor).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - -- - - - -");
        t1.send(ConnectionEvent::EstablishRendPoint(0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(ConnectionEvent::OpenStream(0, node5, 0)).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(ConnectionEvent::FetchFromDirectory).unwrap();
        thread::sleep(time::Duration::from_millis(4000));

        println!(" - - - - - - -");
        t1.send(ConnectionEvent::Introduce1(0)).unwrap();
        thread::sleep(time::Duration::from_millis(30000));

        println!(" - - - - - - -");
        let relay_payload = RelayPayload::new_data_payload("Hello!".as_bytes(), 3);
        let cell = Cell::new_relay_cell(0, relay_payload);
        t1.send(ConnectionEvent::SendCell(cell)).unwrap();

        loop {}
    }
}
