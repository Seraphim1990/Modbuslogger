
use crate::messages::requests::{
    node_request::NodeRequest,
    device_request::DeviceRequest,
    value_request::ValueRequest,
    measure_request::MeasureRequest,
    user_request::UserRequest,
    group_request::GroupRequest,
    sub_group_request::SubGroupsRequest,
};

pub enum Request{
    GetNode(NodeRequest),
    GetDevice(DeviceRequest),
    GetValue(ValueRequest),
    GetDecodingType,
    GetMeasure(MeasureRequest),
    GetUser(UserRequest),
    GetGroup(GroupRequest),
    GetSubGroup(SubGroupsRequest),
}