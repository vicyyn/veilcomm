fn main() {
    println!("Hello, world!");
}
// if main_node.port == 8000 {
//     let destination = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8001);
//     connect_to_peer(destination, events_sender.clone());
// }
// if main_node.port == 8001 {
//     let public_key_bytes = keys.dh.public_key().to_vec();
//     let next_node = Node::new(Ipv4Addr::new(127, 0, 0, 1), 8002);
//     let extend_payload = ExtendPayload::new(next_node, &public_key_bytes);
//     let extend_cell = Cell::new_extend_cell(0, extend_payload);
//     connection.write(extend_cell);
// }
