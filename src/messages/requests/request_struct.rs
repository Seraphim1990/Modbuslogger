
use crate::messages::requests::{
    node_request::NodeRequest,
    device_request::DeviceRequest,
    value_request::ValueRequest,
};

pub enum Request{
    GetNode(NodeRequest),
    GetDevice(DeviceRequest),
    GetValue(ValueRequest),
    GetDecodingType,
    GetMeasure,

    GetLogicGroup
}