use crate::reader::structs::modbus_measure::ModbusMeasure;

#[derive(Debug, Clone, Copy)]
pub enum RegType {
    Coils,
    Discrete,
    Holding,
    Input
}

impl RegType {
    pub fn check_type(reg_type: &str) -> RegType {
        match reg_type {
            "coils" => RegType::Coils,
            "discrete" => RegType::Discrete,
            "holding" => RegType::Holding,
            "input" => RegType::Input,
            _ => RegType::Coils
        }
    }
}
impl Default for RegType {
    fn default() -> RegType {
        RegType::Holding
    }
}

pub trait ValueInterface {
    fn init(&mut self, settings: String, id: i32, logging: bool) -> Vec<i32>;
    fn find_your_registers(&mut self, dataset: &Vec<i32>) -> bool;
    fn get_value(&self, reg_list: &Vec<u16>, timestamp: u64) -> ModbusMeasure;
    fn fail(&self, timestamp: u64) -> ModbusMeasure;
    fn get_type(&self) -> RegType;

    fn get_id(&self) -> i32;
}