use tokio::sync::oneshot;
use crate::db::schemas::user_groups::{UserGroupRead, UiUserGroupRead};

pub enum GroupRequest {
    GetAll(GroupGetAll),
    GetById(GroupGetByGroupId),
    GetByUserId(GetByUserId),
    GetForUi(UiUserGroupsRequest)
}

pub struct GroupGetAll  {
    pub request_channel: oneshot::Sender<Result<Vec<UserGroupRead>, ()>>,
}

pub struct GroupGetByGroupId {
    pub group_id: i32,
    pub request_channel: oneshot::Sender<Result<Option<UserGroupRead>, ()>>,
}

pub struct GetByUserId  {
    pub user_id: i32,
    pub request_channel: oneshot::Sender<Result<Vec<UserGroupRead>, ()>>,
}

pub struct UiUserGroupsRequest {
    pub id: i32,
    pub request_channel: oneshot::Sender<Result<Option<UiUserGroupRead>, ()>>,
}