use std::fs;
use std::sync::Arc;
use sqlx::{MySql, MySqlPool, Pool};

use serde::Deserialize;
use sqlx::mysql::MySqlPoolOptions;


#[derive(Debug, Deserialize)]
struct DBConf{
    db_user: String,
    db_password: String,
    db_address: String,
    db_port: String,
    db_name: String,
}

pub async fn init_db(pool_size: u32) -> Pool<MySql> {
    let config_contents = fs::read_to_string("configs/db.toml")
        .expect("Не вдалося прочитати файл db.toml");

    let config: DBConf = toml::from_str(config_contents.as_str())
        .expect("Невдалось десеріалізувати");
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.db_user, config.db_password, config.db_address, config.db_port, config.db_name
    );
    MySqlPoolOptions::new()
        .max_connections(pool_size)
        .connect(&url)
        .await
        .expect("Не вдалося підключитися до БД")
}