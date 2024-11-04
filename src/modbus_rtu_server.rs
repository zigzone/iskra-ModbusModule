use once_cell::sync::Lazy;
use rmodbus::server::ModbusFrame;
use rmodbus::{ModbusFrameBuf, ModbusProto};
use serial::prelude::*;
use std::io::{Read, Write};
use std::sync::RwLock;
use std::time::Duration;
use crate::yaml_configurator::RS485ModbusCFG;
use crate::ModbusStCustomSize;

pub fn listen(/*unit: u8, port: &str*/ config: RS485ModbusCFG, registers:&Lazy<RwLock<ModbusStCustomSize>>) {
    //let mut port = serial::open(&config.dev);
    let mut port = match serial::open(&config.dev) {
        Ok(ttyPort) => {
            println!("using {} OK", &config.dev);
            ttyPort
        },
        Err(error) => {
            println!("using {} ERR", &config.dev);
            std::process::exit(11);
        },
    };
    
    port.reconfigure(&|settings| {
        (settings.set_baud_rate(config.options.baud_rate).unwrap());
        settings.set_char_size(config.options.char_size);
        settings.set_parity(config.options.parity);
        settings.set_stop_bits(config.options.stop_bits);
        settings.set_flow_control(config.options.flow_control);
        Ok(())
    })
    .unwrap();
    port.set_timeout(Duration::from_secs(10000)).unwrap();
    let mut buf: ModbusFrameBuf = [0; 256];
    let mut response = Vec::new();
    loop {
        if port.read(&mut buf).unwrap() > 0 {
            println!("got frame");
        };
        let mut frame = ModbusFrame::new(config.unit, &buf, ModbusProto::Rtu, &mut response);
        if frame.parse().is_err() {
            println!("server error");
            return;
        }
        if frame.processing_required {
            let result = if frame.readonly {
                frame.process_read(&*registers.read().unwrap())
            } else {
                frame.process_write(&mut *registers.write().unwrap())
            };
            if result.is_err() {
                println!("frame processing error");
                return;
            }
        }
        if frame.response_required {
            frame.finalize_response().unwrap();
            println!("{:x?}", response);
            port.write_all(response.as_slice()).unwrap();
        }
    }
}