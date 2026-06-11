use tokio::sync::oneshot;
use crate::db::schemas::device::DeviceRead;

pub enum DeviceRequest {
    GetDeviceById(GetDeviceById),
    GetByNode(GetDeviceByNode),
    GetAllDevices(GetAllDevices),
    GetDeleted(GetDeleted),
}

pub struct GetDeviceById {
    pub id: i32,
    pub request_channel: oneshot::Sender<Result<Option<DeviceRead>, ()>>,
}

pub struct GetDeviceByNode {
    pub node_id: i32,
    pub request_channel: oneshot::Sender<Result<Vec<DeviceRead>, ()>>,
}

pub struct GetAllDevices {
    pub request_channel: oneshot::Sender<Result<Vec<DeviceRead>, ()>>,
}

pub struct GetDeleted {
    pub request_channel: oneshot::Sender<Result<Vec<DeviceRead>, ()>>,
}