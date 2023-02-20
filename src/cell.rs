// The basic unit of communication for onion routers and onion
// proxies is a fixed-width "cell". 512 bytes size.

const CELL_PAYLOAD_SIZE: usize = 509;

pub struct Cell {
    pub circ_id: u16,
    pub command: u8,
    pub payload: [u8; CELL_PAYLOAD_SIZE],
}
