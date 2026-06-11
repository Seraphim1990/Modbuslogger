use crate::db::schemas::device::{DeviceCreate, DeviceUpdate, DeviceDelete};
pub enum DeviceCommand{
    Create(DeviceCreate),
    Delete(DeviceDelete),
    Update(DeviceUpdate),
}