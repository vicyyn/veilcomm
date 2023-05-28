use core::time;
use std::{net::SocketAddrV4, sync::mpsc::Sender, thread};

use crate::TorEvent;

pub fn create_circuit(
    circ_id: u16,
    t: Sender<TorEvent>,
    first_hop: SocketAddrV4,
    second_hop: SocketAddrV4,
    third_hop: SocketAddrV4,
) {
    println!(" - -- - - - -");
    t.send(TorEvent::FetchFromDirectory).unwrap();
    thread::sleep(time::Duration::from_millis(4000));

    println!(" - -- - - - -");
    t.send(TorEvent::Connect(first_hop)).unwrap();
    thread::sleep(time::Duration::from_millis(1000));

    println!(" - -- - - - -");
    t.send(TorEvent::SendCreate(circ_id, first_hop)).unwrap();
    thread::sleep(time::Duration::from_millis(1000));

    println!(" - -- - - - -");
    t.send(TorEvent::SendExtend(circ_id, second_hop)).unwrap();
    thread::sleep(time::Duration::from_millis(4000));

    println!(" - -- - - - -");
    t.send(TorEvent::SendExtend(circ_id, third_hop)).unwrap();
    thread::sleep(time::Duration::from_millis(4000));
}
