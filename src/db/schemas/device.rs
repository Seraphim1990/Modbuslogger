use serde::{Serialize, Deserialize};
use sqlx::FromRow;

/*
  `id` int NOT NULL AUTO_INCREMENT,
  `parent_node_id` int NOT NULL,
  `device_name` varchar(100) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `address` int NOT NULL,
  `time_for_recall` int NOT NULL,
  `timeout` int NOT NULL,
  `retry_count` int NOT NULL,
  `is_active` tinyint(1) NOT NULL,
  `read_by_group` tinyint(1) NOT NULL,
  `description` text COLLATE utf8mb4_unicode_ci,
  `deleted` tinyint(1) DEFAULT '0',
  `deleted_at` bigint DEFAULT NULL,
 */

#[derive(Debug, Deserialize)]
pub struct DeviceCreate {
    pub device_name: String,
    pub address: i32,
    pub parent_node_id: i32,
    pub time_for_recall: i32,
    pub timeout: i32,
    pub retry_count: i32,
    pub is_active: bool,
    pub read_by_group: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DeviceRead {
    pub id: i32,
    pub device_name: Option<String>,
    pub address: i32,
    pub parent_node_id: i32,
    pub time_for_recall: i32,
    pub timeout: i32,
    pub retry_count: i32,
    pub is_active: bool,
    pub read_by_group: bool,
    pub description: Option<String>,
    pub deleted: Option<bool>,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceUpdate {
    pub id: i32,
    pub device_name: Option<String>,
    pub address: Option<i32>,
    pub parent_node_id: Option<i32>,
    pub time_for_recall: Option<i32>,
    pub timeout: Option<i32>,
    pub retry_count: Option<i32>,
    pub is_active: Option<bool>,
    pub read_by_group: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceDelete {
    pub id: i32,
}