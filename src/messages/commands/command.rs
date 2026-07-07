use std::sync::Arc;
use tokio::sync::oneshot;
use crate::db::schemas::tokens::RefreshToken;
use crate::messages::commands::{
    node::NodeCommand,
    device::DeviceCommand,
    value::ValueCommand,
    users::UserCommand,
    groups::GroupCommand,
    sub_groups::SubGroupCommand,
    asign::AssignGroupsAndValuesCommand,
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
    UserCommand(Arc<UserCommand>),
    GroupCommand(Arc<GroupCommand>),
    SubGroupCommand(Arc<SubGroupCommand>),
    AssignGroupsAndValuesCommand(Arc<AssignGroupsAndValuesCommand>),
    TokenUpdate(Arc<RefreshToken>)
}