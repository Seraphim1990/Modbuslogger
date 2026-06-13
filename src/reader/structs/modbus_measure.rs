use std::sync::Arc;

#[derive(Debug)]
pub struct ModbusMeasure{
    pub measure_value: f64,
    pub is_logging: bool,
    pub value_id: u32,
    pub tag: Arc<String>,
    pub measure_time: u64
}

impl ModbusMeasure{
    pub fn default() -> ModbusMeasure{
        ModbusMeasure {
            measure_value: 0.0,
            is_logging: false,
            value_id: 0,
            tag: Arc::new(String::new()),
            measure_time: 0
        }
    }
}

