// 1. create communication
// generate publickey
// choose introduction points -> advertise on lookup service
// build circuit for each introduction point
// ESTABLISH_INTRO =>
//
// 2.
// connect to lookup service
// choose OR as rendezvous point (RP) and build the circuit
// open a stream with one of the introduction points
// send encrypted message containing the RP, rendezvous cookie, half of the dh handshake and information about the sender
// if connection is accepted, build circuit to RP, send half of the dh handshake, hash of key
