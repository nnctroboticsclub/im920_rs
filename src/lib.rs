#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::time::Duration;
use srobo_base::communication::{AsyncReadableStream, WritableStream};
use srobo_base::parser;
use srobo_base::time::TimeImpl;
use srobo_base::utils::fifo::{self, Spsc, SpscRx, SpscTx};
use srobo_base::utils::lined::Lined;
use srobo_base::utils::swmr::{Swmr, SwmrReader, SwmrWriter};

#[derive(Debug, Clone, Copy)]
enum LineMarker {
    Version,
    NodeNumber,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IM920Result {
    Ok,
    Ng,
}

#[derive(Debug)]
pub enum IM920Error<E> {
    SerialError(E),
    Fifo(fifo::Error),
    Timeout,
    OperationFailed,
}

#[derive(Debug)]
pub struct IM920Rx {
    pub rssi: u8,
    pub packet: IM920Packet,
}

#[derive(Debug)]
pub struct IM920Packet {
    pub node_id: u16,
    pub data: Vec<u8>,
}

type DataCallback = Box<dyn Fn(IM920Rx) -> ()>;

pub struct IM920<E, S: WritableStream<Error = E>, Time: TimeImpl> {
    dev_tx: S,

    mode_tx: SpscTx<LineMarker, 8>,

    node_number: SwmrReader<Option<u16>>,
    version: SwmrReader<Option<String>>,
    result_rx: SpscRx<IM920Result, 4>,
    unknown_lines_rx: SpscRx<String, 8>,

    on_data_cb: SwmrWriter<Option<DataCallback>>,

    time: Time,

    p: PhantomData<E>,
}

impl<E, S: WritableStream<Error = E>, Time: TimeImpl> IM920<E, S, Time> {
    pub fn new(dev_tx: S, mut dev_rx: impl AsyncReadableStream, time: Time) -> IM920<E, S, Time> {
        let (mode_tx, mode_rx) = Spsc::new();

        let (nn_tx, nn_rx) = Swmr::new(None);
        let (ver_tx, ver_rx) = Swmr::new(None);
        let (on_data_tx, on_data_rx) = Swmr::<Option<DataCallback>>::new(None);
        let (result_tx, result_rx) = Spsc::new();
        let (unknown_lines_tx, unknown_lines_rx) = Spsc::new();

        let lined = Box::into_raw(Box::new(Lined::new()));

        dev_rx
            .on_data(Box::new(move |data| {
                let lined = unsafe { &mut *lined };

                lined.feed(data);

                while let Some(data) = lined.get_line() {
                    if data.len() > 2 && data[..3] == [48, 48, 44] {
                        let node_id = &data[3..7];
                        let node_id = parser::u16(node_id).unwrap().1;

                        let rssi = &data[8..10];
                        let rssi = parser::u8(rssi).unwrap().1;

                        let data = &data[11..];
                        let data = parser::comma_separated_u8(data, 13 /* \r */).unwrap().1;

                        let message = IM920Rx {
                            rssi,
                            packet: IM920Packet { node_id, data },
                        };

                        if let Some(ref cb) = *on_data_rx {
                            cb(message);
                        }
                    } else {
                        let line = String::from_utf8(data.to_vec()).unwrap();
                        match mode_rx.dequeue() {
                            Some(LineMarker::Version) => {
                                ver_tx.write(Some(line));
                            }
                            Some(LineMarker::NodeNumber) => {
                                let node_number = parser::u16(&data).unwrap().1;
                                nn_tx.write(Some(node_number));
                            }
                            Some(LineMarker::Result) => {
                                let result = match line.as_str() {
                                    "OK\r\n" => IM920Result::Ok,
                                    "NG\r\n" => IM920Result::Ng,
                                    _ => panic!("Unknown result: {:?}", line),
                                };
                                result_tx.enqueue(result).expect("Failed to enqueue result");
                            }
                            None => {
                                unknown_lines_tx
                                    .enqueue(line)
                                    .expect("Failed to enqueue line");
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
            version: ver_rx,
            result_rx,
            unknown_lines_rx,
            on_data_cb: on_data_tx,
            time,
            p: PhantomData,
        }
    }

    pub fn on_data(&mut self, cb: DataCallback) {
        self.on_data_cb.write(Some(cb));
    }

    pub fn get_node_number(&mut self, timeout: Duration) -> Result<u16, IM920Error<E>> {
        if self.node_number.is_some() {
            return Ok(self.node_number.unwrap());
        }

        self.mode_tx
            .enqueue(LineMarker::NodeNumber)
            .map_err(|e| IM920Error::Fifo(e))?;
        self.dev_tx
            .write(b"RDNN\r\n")
            .map_err(|e| IM920Error::SerialError(e))?;

        if self.node_number.wait_available(timeout, &self.time) {
            Ok(self.node_number.unwrap())
        } else {
            Err(IM920Error::Timeout)
        }
    }

    pub fn get_version(&mut self, timeout: Duration) -> Result<String, IM920Error<E>> {
        if self.version.is_some() {
            return Ok(<Option<String> as Clone>::clone(&self.version)
                .unwrap()
                .clone());
        }

        self.mode_tx
            .enqueue(LineMarker::Version)
            .map_err(|e| IM920Error::Fifo(e))?;
        self.dev_tx
            .write(b"RDVR\r\n")
            .map_err(|e| IM920Error::SerialError(e))?;

        if self.version.wait_available(timeout, &self.time) {
            Ok(<Option<String> as Clone>::clone(&self.version)
                .unwrap()
                .clone())
        } else {
            Err(IM920Error::Timeout)
        }
    }

    fn get_result(&mut self, timeout: Duration) -> Result<IM920Result, IM920Error<E>> {
        if self.result_rx.len() > 0 {
            return Ok(self.result_rx.dequeue().unwrap().clone());
        }

        if self.result_rx.wait_available(timeout, &self.time) {
            Ok(self.result_rx.dequeue().unwrap().clone())
        } else {
            Err(IM920Error::Timeout)
        }
    }

    pub fn transmit_delegate(
        &mut self,
        packet: IM920Packet,
        timeout: Duration,
    ) -> Result<(), IM920Error<E>> {
        self.mode_tx
            .enqueue(LineMarker::Result)
            .map_err(|e| IM920Error::Fifo(e))?;

        self.dev_tx
            .write(
                format!(
                    "TXDG{:04X},{}\r\n",
                    packet.node_id,
                    packet
                        .data
                        .iter()
                        .map(|x| format!("{:02X}", x))
                        .collect::<Vec<String>>()
                        .join(",")
                )
                .as_bytes(),
            )
            .map_err(|e| IM920Error::SerialError(e))?;

        match self.get_result(timeout) {
            Ok(IM920Result::Ok) => Ok(()),
            Ok(IM920Result::Ng) => Err(IM920Error::OperationFailed),
            Err(e) => Err(e),
        }
    }
}
