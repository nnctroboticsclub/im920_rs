use crate::packet::Packet;

#[derive(Debug)]
pub struct RxData {
    pub rssi: u8,
    pub packet: Packet,
}
