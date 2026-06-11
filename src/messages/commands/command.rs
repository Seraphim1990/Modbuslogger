use std::sync::Arc;
use tokio::sync::oneshot;
use crate::messages::commands::{
    node::NodeCommand,
    device::DeviceCommand,
    value::ValueCommand
};

pub struct Command{
    pub cmd: CommandType,
    pub request_channel: oneshot::Sender<Result<(), String>>,
}

#[derive(Clone)]
pub enum CommandType{
    NodeCommand(Arc<NodeCommand>),
    DeviceCommand(Arc<DeviceCommand>),
    ValueCommand(Arc<ValueCommand>),
}