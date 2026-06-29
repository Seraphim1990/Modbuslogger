use crate::db::schemas::users::{UserRead, LoginRequest};
use tokio::sync::oneshot::Sender;

pub enum UserRequest {
    GetById(UserGetById),
    GetAll(UserGetAll),
    Verify(GetVerifyRequest),
}
pub struct UserGetById  {
    pub id: i32,
    pub request_channel: Sender<Result<Option<UserRead>, ()>>
}

pub struct UserGetAll  {
    pub request_channel: Sender<Result<Vec<UserRead>, ()>>,
}

pub struct GetVerifyRequest {
    pub user: LoginRequest,
    pub request_channel: Sender<Result<bool, ()>>,
}
