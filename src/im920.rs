use core::{marker::PhantomData, time::Duration};

use alloc::{boxed::Box, format, string::String, vec::Vec};
use srobo_base::{
    communication::{AsyncReadableStream, WritableStream},
    parser,
    time::TimeImpl,
    utils::{
        fifo::{Spsc, SpscRx, SpscTx},
        lined::Lined,
        string_queue::StringQueue,
        swmr::{Swmr, SwmrReader, SwmrWriter},
    },
};

use crate::{
    error::Error, line_marker::LineMarker, packet::Packet, result::IM920Result, rx_data::RxData,
};

type DataCallback = Box<dyn Fn(RxData) -> ()>;

pub struct IM920<'a, E, S: WritableStream<Error = E>, Time: TimeImpl> {
    dev_tx: &'a mut S,

    mode_tx: SpscTx<LineMarker, 8>,

    node_number: SwmrReader<Option<u16>>,
    group_number: SwmrReader<Option<u32>>,
    channel: SwmrReader<Option<u8>>,
    version: SwmrReader<[u8; 32]>,
    result_rx: SpscRx<IM920Result, 4>,

    on_data_cb: SwmrWriter<Option<DataCallback>>,

    time: &'a Time,

    p: PhantomData<E>,
}

impl<'a, E, S: WritableStream<Error = E>, Time: TimeImpl> IM920<'a, E, S, Time> {
    pub fn new(
        dev_tx: &'a mut S,
        dev_rx: &'a mut impl AsyncReadableStream,
        time: &'a Time,
    ) -> IM920<'a, E, S, Time> {
        let (mode_tx, mode_rx) = Spsc::new();

        let (nn_tx, nn_rx) = Swmr::new(None);
        let (gn_tx, gn_rx) = Swmr::new(None);
        let (ch_tx, ch_rx) = Swmr::new(None);

        let (ver_tx, ver_rx) = Swmr::new([0; 32]);
        let (on_data_tx, on_data_rx) = Swmr::<Option<DataCallback>>::new(None);
        let (result_tx, result_rx) = Spsc::new();
        let (unknown_lines_tx, _unknown_lines_rx) = StringQueue::<64, 2>::new();

        let lined = Box::into_raw(Box::new(Lined::new()));
        let rx_buffer = Box::into_raw(Box::new([0; 128]));

        dev_rx
            .on_data(Box::new(move |data| {
                let rx_buffer = unsafe { &mut *rx_buffer };
                let lined = unsafe { &mut *lined };

                lined.feed(data).expect("Failed to feed data");

                while let Some(data) = lined.get_line() {
                    if data.len() > 2 && data[..3] == [48, 48, 44] {
                        let node_id = &data[3..7];
                        let node_id = match parser::u16(node_id) {
                            Ok((_, node_id)) => node_id,
                            _ => continue,
                        };

                        let rssi = &data[8..10];
                        let rssi = match parser::u8(rssi) {
                            Ok((_, rssi)) => rssi,
                            _ => continue,
                        };

                        let data = &data[11..];
                        let len =
                            match parser::comma_separated_u8(data, 13 /* \r */, rx_buffer) {
                                Ok((_, len)) => len,
                                _ => continue,
                            };

                        let message = RxData {
                            rssi,
                            packet: Packet {
                                node_id,
                                data: &rx_buffer[..len],
                            },
                        };

                        if let Some(ref cb) = *on_data_rx {
                            cb(message);
                        }
                    } else {
                        match mode_rx.dequeue() {
                            Some(LineMarker::Version) => {
                                let dest = ver_tx.as_mut();
                                for (i, ch) in data.iter().enumerate().rev() {
                                    dest[i] = *ch;
                                }
                            }
                            Some(LineMarker::NodeNumber) => {
                                let node_number = match parser::u16(&data) {
                                    Ok((_, node_number)) => node_number,
                                    _ => continue,
                                };
                                let _ = nn_tx.write(Some(node_number));
                            }
                            Some(LineMarker::GroupNumber) => {
                                let group_number = match parser::u32(&data) {
                                    Ok((_, group_number)) => group_number,
                                    _ => continue,
                                };
                                let _ = gn_tx.write(Some(group_number));
                            }
                            Some(LineMarker::Result) => {
                                let result = match data[0] {
                                    b'O' => IM920Result::Ok,
                                    b'N' => IM920Result::Ng,
                                    _ => continue,
                                };
                                let _ = result_tx.enqueue(result);
                            }
                            Some(LineMarker::Channel) => {
                                let channel = match parser::u8(&data) {
                                    Ok((_, channel)) => channel,
                                    _ => continue,
                                };
                                let _ = ch_tx.write(Some(channel));
                            }
                            None => {
                                let _ = unknown_lines_tx.enqueue(data);
                            }
                        }
                    }
                }
            }))
            .expect("Failed to register callback");

        IM920 {
            dev_tx: dev_tx,
            mode_tx,
            node_number: nn_rx,
            group_number: gn_rx,
            channel: ch_rx,
            version: ver_rx,
            result_rx,
            on_data_cb: on_data_tx,
            time,
            p: PhantomData,
        }
    }

    pub fn on_data(&mut self, cb: DataCallback) {
        self.on_data_cb.write(Some(cb));
    }

    pub fn get_node_number(&mut self, timeout: Duration) -> Result<u16, Error<E>> {
        if self.node_number.is_some() {
            return Ok(self.node_number.unwrap());
        }

        self.mode_tx
            .enqueue(LineMarker::NodeNumber)
            .map_err(|e| Error::Fifo(e))?;
        self.dev_tx
            .write(b"RDNN\r\n")
            .map_err(|e| Error::SerialError(e))?;

        if self.node_number.wait_available(timeout, self.time) {
            Ok(self.node_number.unwrap())
        } else {
            Err(Error::Timeout)
        }
    }

    pub fn get_channel(&mut self, timeout: Duration) -> Result<u8, Error<E>> {
        if self.channel.is_some() {
            return Ok(self.channel.unwrap());
        }

        self.mode_tx
            .enqueue(LineMarker::Channel)
            .map_err(|e| Error::Fifo(e))?;

        self.dev_tx
            .write(b"RDCH\r\n")
            .map_err(|e| Error::SerialError(e))?;

        if self.channel.wait_available(timeout, self.time) {
            Ok(self.channel.unwrap())
        } else {
            Err(Error::Timeout)
        }
    }

    pub fn get_group_number(&mut self, timeout: Duration) -> Result<u32, Error<E>> {
        if self.group_number.is_some() {
            return Ok(self.group_number.unwrap());
        }

        self.mode_tx
            .enqueue(LineMarker::GroupNumber)
            .map_err(|e| Error::Fifo(e))?;
        self.dev_tx
            .write(b"RDGN\r\n")
            .map_err(|e| Error::SerialError(e))?;

        if self.group_number.wait_available(timeout, self.time) {
            Ok(self.group_number.unwrap())
        } else {
            Err(Error::Timeout)
        }
    }

    pub fn get_version(&mut self, timeout: Duration) -> Result<&str, Error<E>> {
        if self.version[0] != 0 {
            return Ok(core::str::from_utf8(&*self.version).unwrap());
        }

        self.mode_tx
            .enqueue(LineMarker::Version)
            .map_err(|e| Error::Fifo(e))?;

        self.dev_tx
            .write(b"RDVR\r\n")
            .map_err(|e| Error::SerialError(e))?;

        if self.version.wait_for(|x| x[0] != 0, timeout, self.time) {
            Ok(core::str::from_utf8(&*self.version).unwrap())
        } else {
            Err(Error::Timeout)
        }
    }

    fn get_result(&mut self, timeout: Duration) -> Result<IM920Result, Error<E>> {
        if self.result_rx.len() > 0 {
            return Ok(self.result_rx.dequeue().unwrap().clone());
        }

        if self.result_rx.wait_available(timeout, self.time) {
            Ok(self.result_rx.dequeue().unwrap().clone())
        } else {
            Err(Error::Timeout)
        }
    }

    pub fn transmit_delegate(&mut self, packet: Packet, timeout: Duration) -> Result<(), Error<E>> {
        self.mode_tx
            .enqueue(LineMarker::Result)
            .map_err(|e| Error::Fifo(e))?;

        self.dev_tx
            .write(
                format!(
                    "TXDG {:04X},{}\r\n",
                    packet.node_id,
                    packet
                        .data
                        .iter()
                        .map(|x| format!("{:02X}", x))
                        .collect::<Vec<String>>()
                        .join("")
                )
                .as_bytes(),
            )
            .map_err(|e| Error::SerialError(e))?;

        match self.get_result(timeout) {
            Ok(IM920Result::Ok) => Ok(()),
            Ok(IM920Result::Ng) => Err(Error::OperationFailed),
            Err(e) => Err(e),
        }
    }

    pub fn enable_write(&mut self, timeout: Duration) -> Result<(), Error<E>> {
        self.mode_tx
            .enqueue(LineMarker::Result)
            .map_err(|e| Error::Fifo(e))?;

        self.dev_tx
            .write("ENWR\r\n".as_bytes())
            .map_err(|e| Error::SerialError(e))?;

        match self.get_result(timeout) {
            Ok(IM920Result::Ok) => Ok(()),
            Ok(IM920Result::Ng) => Err(Error::OperationFailed),
            Err(e) => Err(e),
        }
    }

    pub fn set_node_number(&mut self, node_number: u16, timeout: Duration) -> Result<(), Error<E>> {
        self.mode_tx
            .enqueue(LineMarker::Result)
            .map_err(|e| Error::Fifo(e))?;

        self.dev_tx
            .write(format!("STNN{node_number:04x}\r\n").as_bytes())
            .map_err(|e| Error::SerialError(e))?;

        match self.get_result(timeout) {
            Ok(IM920Result::Ok) => Ok(()),
            Ok(IM920Result::Ng) => Err(Error::OperationFailed),
            Err(e) => Err(e),
        }
    }

    pub fn set_channel(&mut self, channel: u8, timeout: Duration) -> Result<(), Error<E>> {
        self.mode_tx
            .enqueue(LineMarker::Result)
            .map_err(|e| Error::Fifo(e))?;

        self.dev_tx
            .write(format!("STCH{channel:02x}\r\n").as_bytes())
            .map_err(|e| Error::SerialError(e))?;

        match self.get_result(timeout) {
            Ok(IM920Result::Ok) => Ok(()),
            Ok(IM920Result::Ng) => Err(Error::OperationFailed),
            Err(e) => Err(e),
        }
    }
}
