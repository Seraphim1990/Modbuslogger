use serde::{Serialize, Deserialize};
use sqlx::FromRow;
/*
  `id` int NOT NULL AUTO_INCREMENT,
  `group_id` int NOT NULL,
  `subgroup_name` varchar(100) NOT NULL,
  `description` text COLLATE utf8mb4_unicode_ci,
  PRIMARY KEY (`id`),
 */

#[derive(Serialize, Deserialize, FromRow)]
pub struct UserSubGroupRead {
    pub id: i32,
    pub group_id: i32,
    pub subgroup_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubGroupCreate {
    pub group_id: i32,
    pub subgroup_name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserSubGroupUpdate {
    pub id: i32,
    pub group_id: Option<i32>,
    pub subgroup_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserSubGroupDelete{
    pub id:i32,
}