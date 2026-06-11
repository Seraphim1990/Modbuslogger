use tokio::sync::oneshot;
use crate::db::schemas::node::NodeRead;

pub enum NodeRequest {
    GetById(GetById),
    GetByIp(GetByIp),
    GetAll(GetAllNodes),
}

pub struct GetById {
    pub node_id: i32,
    pub request_channel: oneshot::Sender<Result<Option<NodeRead>, ()>>,
}

pub struct GetAllNodes {
    pub request_channel: oneshot::Sender<Result<Vec<NodeRead>, ()>>,
}

pub struct GetByIp {
    pub node_ip: String,
    pub request_channel: oneshot::Sender<Result<Option<NodeRead>, ()>>,
}
