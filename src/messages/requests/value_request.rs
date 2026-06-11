use tokio::sync::oneshot;
use crate::db::schemas::node::NodeRead;
use crate::db::schemas::value_unit::ValueRead;
pub enum ValueRequest {
    GetValue(GetValue),
    GetAll(GetAllValues),
    GetByDeviceId(GetByDeviceId),
    GetByGroup(GetByGroup),
    GetLoggingOnly(GetLoggingOnly),
}

pub struct GetValue{
    pub id: i32,
    pub request_channel: oneshot::Sender<Result<Option<ValueRead>, ()>>,
}

pub struct GetAllValues {
    pub request_channel: oneshot::Sender<Result<Vec<ValueRead>, ()>>,
}


pub struct GetByDeviceId{
    pub device_id: i32,
    pub request_channel: oneshot::Sender<Result<Vec<ValueRead>, ()>>,
}

pub struct GetByGroup {
    pub group_id: i32,
    pub request_channel: oneshot::Sender<Result<Vec<ValueRead>, ()>>,
}

pub struct GetLoggingOnly {
    pub request_channel: oneshot::Sender<Result<Vec<ValueRead>, ()>>,
}

