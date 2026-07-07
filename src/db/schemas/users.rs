use serde::{Serialize, Deserialize};
use sqlx::FromRow;

/*
  `id` int NOT NULL AUTO_INCREMENT,
  `username` varchar(100) NOT NULL UNIQUE,
  `password_hash` varchar(255) NOT NULL,
  `role_id` int NOT NULL,
  `is_active` tinyint(1) NOT NULL DEFAULT '1',
 */

#[derive(Serialize, Deserialize, FromRow)]
pub struct UserRead {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub role_id: i32,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password_raw: String, // Пароль у чистому вигляді, який ввів користувач
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdate {
    pub id: i32,
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub role_id: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreate {
    pub username: String,
    pub password_hash: String,
    pub role_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct UserDelete{
    pub id:i32
}