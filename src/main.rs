use core::str;
use std::io;
use serial::PortSettings;
use std::str::FromStr;
use rmodbus::server::context::ModbusContext;
use std::net::{SocketAddr, TcpListener, UdpSocket};
use chrono;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use rmodbus::server::storage::ModbusStorage;
mod yaml_configurator;
mod modbus_rtu_server;
mod modbus_tcp_server;
const CONFIG_NAME: &str = "config.yaml";

fn main()->std::io::Result<()> {
    println!("server starting in: {:?}", chrono::offset::Local::now());
    // let modbus_rs485_rtu_server;
    let configs = yaml_configurator::getYamlConfigs(CONFIG_NAME);
    let mut modbus_registers = &MODBUS_REGISTERS;
    let metric_stream = std::thread::spawn(move ||{
        udp_modbus_update(move |datagram:&str|{
           // println!("metric datagramma: {datagram}");
            let metric = parse_metric(datagram);
            match metric {
                MetricOpt::metric { ch, value, status } => {
                    //println!("reg_num: {} value: {} status: {}",(ch+1)/2, value, status);
                    match modbus_registers.write(){
                        Ok(mut mb_reg) => {
                            let metric_reg_adr:u16  = (ch*2-2).into();
                            let status_reg_adr:u16 = (20+ch-1).into();
                            mb_reg.set_inputs_from_f32(metric_reg_adr, value);
                            mb_reg.set_input(status_reg_adr, status.into());
                            println!("reg_num: {} value: {} status: {}",metric_reg_adr, value, status);
                        },
                        Err(_) => todo!(),
                    };
                },
                MetricOpt::Err()=>{
                    println!("invalid incomming datagram");
                }  
            };
        }, &configs.sources_data.metric_socket)
    });

    let status_stream = std::thread::spawn(move ||{
        udp_modbus_update(|datagram:&str|{
            println!("status datagramma: {datagram}")
        }, &configs.sources_data.status_socket)
    });

    let event_stream = std::thread::spawn(move ||{
        udp_modbus_update(|datagram:&str|{
            println!("event datagramma: {datagram}")
        }, &configs.sources_data.event_socket)
    });
    let modbus_tcp_server = std::thread::spawn(move ||{
        let addr = match  configs.modbus_tcp.tcp_socket.parse::<SocketAddr>(){
            Ok(socket_conf) => socket_conf,
            Err(err_message) => panic!("Возникла проблема: {}", err_message)
        };
        let listener = match TcpListener::bind(addr){
            Ok(done_listener) => {
                println!("using {} OK", addr);
                done_listener
            },
            Err(error) => {
                println!("using {} ERR", addr);
                std::process::exit(12);
            }
        };
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            std::thread::spawn(move|| {
                modbus_tcp_server::listen(configs.modbus_tcp.unit_id, stream, &mut modbus_registers);
            });
        };
    });
    let modbus_rs485_rtu_server = std::thread::spawn(move ||{
        let modbus_rtu_cfg = yaml_configurator::RS485ModbusCFG{
            dev:configs.modbus_rtu.device,
            unit:configs.modbus_rtu.unit_id,
            options:PortSettings{
                baud_rate :serial::Baud9600,
                char_size :serial::Bits8,
                parity :serial::ParityNone,
                stop_bits :serial::Stop1,
                flow_control: serial::FlowNone
            }
        };
        modbus_rtu_server::listen(modbus_rtu_cfg, &mut modbus_registers);
    });
    modbus_rs485_rtu_server.join().unwrap();
    modbus_tcp_server.join().unwrap();
    metric_stream.join().unwrap();
    status_stream.join().unwrap();
    event_stream.join().unwrap();
    Ok(())
}


pub type ModbusStCustomSize = ModbusStorage<32, 32, 32, 32>;
static MODBUS_REGISTERS: Lazy<RwLock<ModbusStCustomSize>> = Lazy::new(<_>::default);

fn udp_modbus_update<F>(modificator:F, addr_str:&str)->io::Result<()>
where F: Fn(&str){
    let udp_socket = UdpSocket::bind(addr_str)?;
    let mut buffer = [0u8; 100];
    while let Ok((len, SocketAddr)) =  udp_socket.recv_from(&mut buffer){
        let date_log = chrono::offset::Local::now();
        let data = &buffer[..len];
        let data_str = str::from_utf8(data).unwrap();
        modificator(&data_str);
    }
    Ok(())
}

enum MetricOpt{
    metric{ch:u8, value:f32, status:u8},
    Err()
}


fn parse_metric(datagram: &str) -> MetricOpt{
    let datagram_parts:Vec<&str> = datagram.split(&['\n',',']).collect();
    if datagram_parts.len() < 4{
        return  MetricOpt::Err();
    }
    let ch_status = match datagram_parts[2].parse::<u8>(){
        Ok(v) => v,
        Err(_) => {
            return  MetricOpt::Err();
        }
    };
    let ch_num = datagram_parts[1].parse::<u8>().unwrap();
    let mut metric_value:f32 = 0.0;
    if ch_num <= 6 {//temp
        metric_value = match f32::from_str(datagram_parts[3]) {
            Ok(val) => {
                if val == std::f32::INFINITY || val == std::f32::NEG_INFINITY  {
                    0.0
                }else{
                    val
                }
            },
            Err(_)=> 0.0,
        };
        let metric = MetricOpt::metric { ch: ch_num, value: metric_value, status: ch_status };
        return metric;
    }else {//vibro
        metric_value = match f32::from_str(datagram_parts[3]) {
            Ok(val) => {
                if val == std::f32::INFINITY || val == std::f32::NEG_INFINITY  {
                    0.0
                }else{
                    val
                }
            },
            Err(_)=> 0.0,
        };
        let metric = MetricOpt::metric { ch: ch_num, value: metric_value, status: ch_status };
        return metric;
    }
}
