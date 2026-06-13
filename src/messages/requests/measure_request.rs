use tokio::sync::oneshot;


#[derive(Clone, Default, sqlx::FromRow)]
pub struct HashedValue {
    pub val: f64,
    pub timestamp: u64,
}

pub struct MeasureResponse {
    pub id: i32,
    pub values: Vec<HashedValue>,
    pub from: u64,
    pub to: u64,
}
pub struct MeasureRequest {
    pub from: u64,
    pub to: u64,
    pub values_id: Vec<i32>,
    pub response_sender: oneshot::Sender<Result<Vec<MeasureResponse>, String>>,
}