use alloc::vec::Vec;

#[derive(Debug)]
pub struct Packet {
    pub node_id: u16,
    pub data: Vec<u8>,
}
