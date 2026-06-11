use std::sync::Arc;
use crate::reader::reader_loop::decoding_plugins::value_interface::{RegType, ValueInterface};
use crate::reader::reader_loop::decoding_plugins::{
    coma_shift::ComaShift,
    bit_in_word::BitInWord,
    satec_double_register::SatecDoubleRegistersInt32
};
use crate::reader::structs::modbus_measure::ModbusMeasure;

pub fn get_plugin(id: i32) -> Result<RegisterDecodingPlugin, Arc<str>>{
    match id {
        1 => Ok(RegisterDecodingPlugin::Shift(ComaShift::default())),
        2 => Ok(RegisterDecodingPlugin::Satec(SatecDoubleRegistersInt32::default())),
        3 => Ok(RegisterDecodingPlugin::Bit(BitInWord::default())),
        _ => {
            Err(Arc::from("Невідомий тип декодування"))
        }
    }
}

pub enum RegisterDecodingPlugin {
    Bit(BitInWord),
    Satec(SatecDoubleRegistersInt32),
    Shift(ComaShift),
}

impl RegisterDecodingPlugin {
    pub fn init(&mut self, settings: String, id: i32, logging: bool) -> Vec<i32> {
        match self {
            RegisterDecodingPlugin::Bit(p) => {p.init(settings, id, logging)},
            RegisterDecodingPlugin::Satec(p) => {p.init(settings, id, logging)},
            RegisterDecodingPlugin::Shift(p) => {p.init(settings, id, logging)},
        }
    }

    pub fn find_your_registers(&mut self, dataset: &Vec<i32>) -> bool {
        match self {
            RegisterDecodingPlugin::Bit(p) => {p.find_your_registers(dataset)},
            RegisterDecodingPlugin::Satec(p) => {p.find_your_registers(dataset)},
            RegisterDecodingPlugin::Shift(p) => {p.find_your_registers(dataset)},
        }
    }
    pub fn get_value(&self, reg_list: &Vec<u16>, timestamp: u64) -> ModbusMeasure {
        match self {
            RegisterDecodingPlugin::Bit(p) => {p.get_value(reg_list, timestamp)},
            RegisterDecodingPlugin::Satec(p) => {p.get_value(reg_list, timestamp)},
            RegisterDecodingPlugin::Shift(p) => {p.get_value(reg_list, timestamp)},
        }
    }

    pub fn get_type(&self) -> RegType {
        match self {
            RegisterDecodingPlugin::Bit(p) => {p.get_type()},
            RegisterDecodingPlugin::Satec(p) => {p.get_type()},
            RegisterDecodingPlugin::Shift(p) => {p.get_type()},
        }
    }
    pub fn fail(&self, timestamp: u64) -> ModbusMeasure {
        match self {
            RegisterDecodingPlugin::Bit(p) => {p.fail(timestamp)},
            RegisterDecodingPlugin::Satec(p) => {p.fail(timestamp)},
            RegisterDecodingPlugin::Shift(p) => {p.fail(timestamp)},
        }
    }

    pub fn get_id(&self) -> i32 {
        match self {
            RegisterDecodingPlugin::Bit(p) => {p.get_id()},
            RegisterDecodingPlugin::Satec(p) => {p.get_id()},
            RegisterDecodingPlugin::Shift(p) => {p.get_id()},
        }
    }
}