# im920_rs

IM920sL Library for Rust

## Example

```rs
extern crate im920_rs;
extern crate srobo_base;

use std::{thread::sleep, time::Duration};

use im920_rs::Packet;
use srobo_base::communication::{AsyncSerial, SerialDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dev = SerialDevice::new("/dev/ttyUSB1".to_string(), 19200);
    let (mut rx, mut tx) = dev.open().unwrap();
    let mut time = srobo_base::time::HostTime::new();

    let mut im920 = im920_rs::IM920::new(&mut tx, &mut rx, &mut time);

    im920.on_data(Box::new(|data| {
        println!("Received data: {:?}", data);
    }));

    let nn = im920.get_node_number(Duration::new(1, 0)).unwrap();
    let gn = im920.get_group_number(Duration::new(1, 0)).unwrap();
    let ch = im920.get_channel(Duration::new(1, 0)).unwrap();
    println!("Node Number: {}", nn);
    println!("Group Number: {}", gn);
    println!("Channel: {}", ch);

    if ch != 0x02 {
        println!("Setting channel to 0x02");
        im920.set_channel(0x02, Duration::new(1, 0)).unwrap();
    }

    loop {
        let buf = [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80] as [u8; 8];

        sleep(Duration::new(1, 0));
        let tx_result = im920.transmit_broadcast(
            Packet {
                node_id: 0,
                data: &buf,
            },
            Duration::new(1, 0),
        );
        match tx_result {
            Ok(_) => println!("Transmitted broadcast packet"),
            Err(_) => println!("Failed to transmit broadcast packet"),
        };
    }
}
```