use core::{ffi::c_void, time::Duration};

use alloc::{boxed::Box, ffi::CString};
use srobo_base::{
    communication::{CStreamRx, CStreamTx},
    time::CTime,
};

use crate::{Packet, IM920};

pub struct CIM920 {
    im920: IM920<'static, (), CStreamTx, CTime>,
}

#[no_mangle]
pub extern "C" fn __ffi_cim920_new(
    tx: *mut CStreamTx,
    rx: *mut CStreamRx,
    time: *mut CTime,
) -> *mut CIM920 {
    let im920 = Box::into_raw(Box::new(CIM920 {
        im920: IM920::new(unsafe { &mut *tx }, unsafe { &mut *rx }, unsafe {
            &mut *time
        }),
    }));
    // let im920 = null_mut() as *mut CIM920;

    im920
}

#[no_mangle]
pub extern "C" fn __ffi_cim920_on_data(
    instance: *mut CIM920,
    cb: extern "C" fn(ctx: *const c_void, from: u16, data: *const u8, len: usize) -> (),
    ctx: *const c_void,
) -> () {
    let mut im920 = unsafe { Box::from_raw(instance) }.im920;
    im920.on_data(Box::new(move |data| {
        cb(
            ctx,
            data.packet.node_id,
            data.packet.data.as_ptr(),
            data.packet.data.len(),
        );
    }));
}

#[no_mangle]
pub extern "C" fn __ffi_cim920_get_node_number(instance: *mut CIM920, duration_secs: f32) -> u16 {
    let mut im920 = unsafe { Box::from_raw(instance) }.im920;
    let node_number = im920.get_node_number(Duration::from_secs_f32(duration_secs));

    match node_number {
        Ok(node_number) => node_number,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn __ffi_cim920_get_version(instance: *mut CIM920, duration_secs: f32) -> *mut i8 {
    let mut im920 = unsafe { Box::from_raw(instance) }.im920;

    let string = match im920.get_version(Duration::from_secs_f32(duration_secs)) {
        Ok(version) => version,
        Err(_) => return core::ptr::null_mut(),
    };

    let string = CString::new(string).unwrap();

    string.into_raw()
}
#[no_mangle]
pub extern "C" fn __ffi_cim920_transmit_delegate(
    instance: *mut CIM920,
    dest: u16,
    data: *const u8,
    len: usize,
    duration_secs: f32,
) -> bool {
    let mut im920 = unsafe { Box::from_raw(instance) }.im920;

    let packet = Packet {
        node_id: dest,
        data: unsafe { core::slice::from_raw_parts(data, len) }.to_vec(),
    };

    match im920.transmit_delegate(packet, Duration::from_secs_f32(duration_secs)) {
        Ok(_) => true,
        Err(_) => false,
    }
}
