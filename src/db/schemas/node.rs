use serde::{Serialize, Deserialize};
use sqlx::FromRow;

/*
  `id` int NOT NULL AUTO_INCREMENT,
  `ip` varchar(100) COLLATE utf8mb4_unicode_ci NOT NULL,
  `port` int DEFAULT NULL,
  `description` text COLLATE utf8mb4_unicode_ci,
 */
#[derive(Debug, Serialize, FromRow)]
pub struct NodeRead {
    pub id: i32,
    pub ip: String,
    pub port: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NodeCreate {
    pub ip: String,
    pub port: Option<i32>,
    pub description: Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct NodeUpdate {
    pub id: i32,
    pub ip: Option<String>,
    pub port: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NodeDelete {
    pub id: i32,
}