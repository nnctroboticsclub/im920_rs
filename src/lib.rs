#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod error;
mod im920;
mod line_marker;
mod packet;
mod result;
mod rx_data;

pub use error::Error;
pub use im920::IM920;
pub use packet::Packet;
pub use result::IM920Result;
pub use rx_data::RxData;

pub mod ffi;
