// 1. create communication
// generate publickey
// choose introduction points -> advertise on lookup service
// build circuit for each introduction point
// ESTABLISH_INTRO => PUBLISH USER DESCRIPTOR
//
// 2.
// connect to lookup service
// choose OR as rendezvous point (RP) and build the circuit
// open a stream with one of the introduction points
// send encrypted message containing the RP, rendezvous cookie, half of the dh handshake and information about the sender
// if connection is accepted, build circuit to RP, send half of the dh handshake, hash of key

use rand::RngCore;

pub fn generate_random_cookie() -> [u8; 20] {
    let mut rng = rand::thread_rng();
    let mut cookie = [0; 20];
    rng.fill_bytes(&mut cookie);
    cookie
}
