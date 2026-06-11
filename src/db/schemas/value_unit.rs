use serde_json::Value;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;


/*
  `id` int NOT NULL AUTO_INCREMENT,
  `parent_device_id` int NOT NULL,
  `value_name` varchar(100) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `value_tag` varchar(100) COLLATE utf8mb4_unicode_ci NOT NULL,
  `description` text COLLATE utf8mb4_unicode_ci,
  `decoding_type` int NOT NULL,
  `settings` json NOT NULL,
  `is_logging` tinyint(1) NOT NULL,
 */
#[derive(Debug, Deserialize)]
pub struct ValueCreate {
    pub parent_device_id: i32,
    pub value_name: String,
    pub value_tag: String,
    pub description: Option<String>,
    pub decoding_type: i32,
    pub settings: Value,
    pub is_logging: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ValueRead {
    pub id: i32,
    pub parent_device_id: i32,
    pub value_name: String,
    pub value_tag: String,
    pub description: Option<String>,
    pub decoding_type: i32,
    pub settings: Value,
    pub is_logging: bool,
}

#[derive(Debug, Deserialize)]
pub struct ValueUpdate {
    pub id: i32,
    pub parent_device_id: Option<i32>,
    pub value_name: Option<String>,
    pub value_tag: Option<String>,
    pub description: Option<String>,
    pub decoding_type: Option<i32>,
    pub settings: Option<Value>,
    pub is_logging: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ValueDelete {
    pub id: i32,
}