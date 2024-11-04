use std::str;
use std::fs;
use serde_yaml;
use serde::Deserialize;
use serial::PortSettings;

#[derive(Debug, Deserialize)]
pub struct sources_streams{
    pub metric_socket:String,
    pub event_socket:String,
    pub status_socket:String
}
#[derive(Debug, Deserialize)]
pub struct modbusTcp{
    pub running:bool,
    pub tcp_socket:String,
    pub unit_id: u8
}
#[derive(Debug, Deserialize)]
pub struct modbusRtu{
    pub running: bool,
    pub device: String,
    pub unit_id: u8
}

#[derive(Debug, Deserialize)]
pub struct YamlConfigs{
    pub sources_data: sources_streams,
    pub modbus_tcp:modbusTcp,
    pub modbus_rtu:modbusRtu
}
pub struct RS485ModbusCFG{
    pub dev: String,
    pub unit: u8,
    pub options: PortSettings
}

pub fn getYamlConfigs(file_name:&str)->YamlConfigs{
    let file_content = match fs::read_to_string(file_name){
        Ok(file) => file,
        Err(err) => {
            println!("open file: {file_name} unable");
            println!("error message: {}", err);
            std::process::exit(1);
        },
    };  // Чтение содержимого файла
    let config:YamlConfigs = match serde_yaml::from_str(&file_content){
        Ok(configs) => configs,
        Err(err) => {
            println!("get configuration: failed");
            println!("error message: {}", err);
            std::process::exit(2);
        },
    };  // Десериализация YAML в структуру
    return config;
}
