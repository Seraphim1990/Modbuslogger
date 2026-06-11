use std::sync::Arc;

#[derive(Clone, Copy)]
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