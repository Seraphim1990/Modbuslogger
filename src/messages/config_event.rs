use std::sync::Arc;
use crate::db::schemas::node::NodeRead;

#[derive(Copy, Clone)]
pub enum ConfigEventType {
    Create,
    Update,
    Delete
}

#[derive(Clone)]
pub struct ConfigEvent {
    pub event_type: ConfigEventType,
    pub data: Arc<NodeRead>,
}