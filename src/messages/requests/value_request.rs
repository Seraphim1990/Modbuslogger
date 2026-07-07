use tokio::sync::oneshot;
use crate::db::schemas::value_unit::ValueRead;
pub enum ValueRequest {
    GetValue(GetValue),
    GetAll(GetAllValues),
    GetByDeviceId(GetByDeviceId),
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

pub struct GetLoggingOnly {
    pub request_channel: oneshot::Sender<Result<Vec<ValueRead>, ()>>,
}

