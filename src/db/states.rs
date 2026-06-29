use std::fs;
use std::sync::Arc;
use sqlx::{MySql, MySqlPool, Pool};
use crate::db::db_init::code_decode::decrypt_string;

use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;


#[derive(Debug, Deserialize, Serialize)]
pub struct DBConf{
    pub db_user: String,
    pub db_password: String,
    pub db_address: String,
    pub db_port: u16,
}

pub async fn init_db(pool_size: u32) -> Pool<MySql> {
    let config_contents = fs::read_to_string("configs/db.toml")
        .expect("Не вдалося прочитати файл db.toml");
    let config = decrypt_string(&config_contents).expect("Помилка файлу конфігурації БД");

    let config: DBConf = toml::from_str(config.as_str())
        .expect("Невдалось десеріалізувати");
    let url = format!(
        "mysql://{}:{}@{}:{}/scada_db_v2",
        config.db_user, config.db_password, config.db_address, config.db_port
    );
    MySqlPoolOptions::new()
        .max_connections(pool_size)
        .connect(&url)
        .await
        .expect("Не вдалося підключитися до БД")
}