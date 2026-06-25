use std::sync::Arc;
use serde::Serialize;
use crate::reader::structs::modbus_measure::ModbusMeasure;
/*
pub struct MeasureCreate {
    pub value_id: i32,
    pub measure_value: f32,
    pub measure_time: Option<u32>,
}
 */

/*
#[derive(Debug)]
pub struct ModbusMeasure{
    pub measure_value: f64,
    pub is_logging: bool,
    pub value_id: u32,
    pub tag: Arc<String>,
    pub measure_time: u64
}
 */
#[derive(Debug, Copy, Clone, Serialize,PartialOrd, PartialEq)]
pub enum DeviceEventType {
    Full,
    SomePart,
    Failed,
}
pub struct  DeviceEvent {
    pub event: DeviceEventType,
    pub id: i32,
    pub measures: Vec<ModbusMeasure>
}
