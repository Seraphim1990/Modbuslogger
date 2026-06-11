use std::sync::Arc;
use crate::reader::structs::modbus_measure::ModbusMeasure;
/*
pub struct MeasureCreate {
    pub value_id: i32,
    pub measure_value: f32,
    pub measure_time: Option<u32>,
}
 */
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
