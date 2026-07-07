use serde::{Serialize, Deserialize};
use sqlx::FromRow;

// ui

use crate::db::schemas::{
    value_unit::ValueRead,
    user_subgroups::UserSubGroupRead
};
#[derive(Serialize, Debug)]
pub struct UiUserSubGroupsRead {
    pub group_state: UserSubGroupRead,
    pub values: Vec<ValueRead>,
}
#[derive(Serialize, Debug)]
pub struct UiUserGroupRead {
    pub id: i32,
    pub sub_groups: Vec<UiUserSubGroupsRead>,
    pub nodes: Vec<i32>,
    pub devices: Vec<i32>,
}
/*
  `id` int NOT NULL AUTO_INCREMENT,
  `group_name` varchar(100) NOT NULL,
  `description` text COLLATE utf8mb4_unicode_ci,
  PRIMARY KEY (`id`)
 */
#[derive(Serialize, Deserialize, FromRow)]
pub struct UserGroupRead {
    pub id: i32,
    pub group_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGroupCreate {
    pub group_name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserGroupUpdate {
    pub id: i32,
    pub group_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserGroupDelete {
    pub id: i32,
}