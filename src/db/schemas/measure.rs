use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasureCreate {
    pub value_id: i32,
    pub measure_value: f64,
    pub measure_time: u64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MeasureRead {
    pub id: i32,
    pub value_id: i32,
    pub measure_value: f64,
    pub measure_time: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasureUpdate {
    pub value_id: Option<i32>,
    pub measure_value: Option<f64>,
    pub measure_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct MeasureDelete {
    pub id: i32,
}