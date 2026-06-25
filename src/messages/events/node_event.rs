use std::sync::Arc;
use serde::Serialize;

#[derive(Clone, Copy, Serialize, PartialOrd, PartialEq)]
pub enum NodeEventType{
    Connected,
    UnConnected,
    Connecting
}
pub struct NodeEvent {
    pub id: i32,
    pub ip: Arc<String>,
    pub event: NodeEventType,
}