use tokio::sync::oneshot;
use crate::db::schemas::tokens::RefreshToken;

pub struct TokensRequest{
    pub token: String,
    pub request_channel: oneshot::Sender<Result<Option<RefreshToken>, ()>>,
}