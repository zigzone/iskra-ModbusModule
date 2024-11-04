use std::{io::{Read, Write}, net::TcpStream, sync::RwLock};
use once_cell::sync::Lazy;
use rmodbus::{server::ModbusFrame, ModbusFrameBuf, ModbusProto};
use crate::ModbusStCustomSize;

pub fn listen(unit: u8, mut socket:TcpStream, registers:&Lazy<RwLock<ModbusStCustomSize>>) ->(){
    println!("Connection established!");
    let mut buffer: ModbusFrameBuf = [0; 256];
    let mut response:Vec<u8> = Vec::new();
    loop {
        // let mut stream_reader = BufReader::new(&mut socket);
        if socket.read(&mut buffer).unwrap_or(0) == 0 {
            return;
        }
        let mut frame = ModbusFrame::new(unit, &buffer, ModbusProto::TcpUdp, &mut response);
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
            println!("{:x?}", response.as_slice());
            if socket.write(response.as_slice()).is_err() {
                return;
            }
        }
    }
}