use tokio::sync::oneshot::Sender;
use crate::db::schemas::user_subgroups::UserSubGroupRead;

pub enum SubGroupsRequest {
    GetByGroupId(GetByGroupId),
    GetById(GetById),
    GetAll(GetAll),
}

pub struct GetByGroupId  {
    pub group_id: i32,
    pub request_channel: Sender<Result<Vec<UserSubGroupRead>, ()>>,
}

pub struct GetById {
    pub id: i32,
    pub request_channel: Sender<Result<Option<UserSubGroupRead>, ()>>,
}

pub struct GetAll {
    pub request_channel: Sender<Result<Vec<UserSubGroupRead>, ()>>,
}