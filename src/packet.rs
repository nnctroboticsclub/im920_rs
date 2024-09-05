use alloc::vec::Vec;

#[derive(Debug)]
pub struct Packet<'a> {
    pub node_id: u16,
    pub data: &'a [u8],
}
