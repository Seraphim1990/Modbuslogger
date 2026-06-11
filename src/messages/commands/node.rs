
use crate::db::schemas::node::{NodeCreate, NodeUpdate, NodeDelete};

pub enum NodeCommand{
    Create(NodeCreate),
    Delete(NodeDelete),
    Update(NodeUpdate),
}

