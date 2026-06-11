use crate::messages::events::{
    device_event::DeviceEvent,
    node_event::NodeEvent,
};
use std::sync::Arc;

#[derive(Clone)]
pub enum Event {
    NodeEvent(Arc<NodeEvent>),
    DeviceEvent(Arc<DeviceEvent>),
}


