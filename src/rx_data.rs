use crate::packet::Packet;

#[derive(Debug)]
pub struct RxData<'a> {
    pub rssi: u8,
    pub packet: Packet<'a>,
}
