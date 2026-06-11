use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::messages::requests::value_request::*;
use crate::db::schemas::value_unit::{ValueCreate, ValueDelete, ValueRead, ValueUpdate};
use crate::logger::printers;
use sqlx::{MySql, Pool};
use crate::messages::commands::command::{Command, CommandType};
use crate::messages::commands::value::ValueCommand;

pub fn command_value(pool: &Pool<MySql>, command: Command) {
    let pool = pool.clone();

    if let CommandType::ValueCommand(value) = command.cmd {
        let tx = command.request_channel;
        match value.as_ref() {
            ValueCommand::Create(_) => {
                let value = value.clone();
                tokio::spawn(async move {
                    if let ValueCommand::Create(value) = value.as_ref() {
                        let res = create_value(&pool, value).await;
                        if tx.send(res).is_err() {
                            let msg = "Помилка відправки калбеку для ValueCommand::Create".to_string();
                            printers::err(msg);
                        }
                    }
                });
            },
            ValueCommand::Delete(_) => {
                let value = value.clone();
                tokio::spawn(async move {
                   if let ValueCommand::Delete(value) = value.as_ref() {
                       let res = delete_value(&pool, value).await;
                       if tx.send(res).is_err() {
                           let msg = format!("Помилка відправки калбеку для ValueCommand::Delete id: {}", value.id);
                           printers::err(msg);
                       }
                   }
                });
            },
            ValueCommand::Update(_) => {
                let value = value.clone();
                tokio::spawn(async move {
                    if let ValueCommand::Update(value) = value.as_ref() {
                        let res = value_update(&pool, value).await;
                        if tx.send(res).is_err() {
                            let msg = format!("Помилка відправки калбеку для ValueCommand::Update id: {}", value.id);
                            printers::err(msg);
                        }
                    }
                });
            },
        }
    }
}

pub fn value_get(pool: &Pool<MySql>, request: ValueRequest) {
    let pool = pool.clone();
    match request {
        ValueRequest::GetValue(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_value_by_id(&pool, request.id).await;
                if tx.send(res).is_err() {
                    let msg = format!("Помилка відправки ValueRequest::GetValue {}", request.id);
                    printers::warn(msg);
                }
            });
        },
        ValueRequest::GetByDeviceId(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_value_by_device_id(&pool, request.device_id).await;
                if tx.send(res).is_err() {
                    let msg = format!("Помилка відправки ValueRequest::GetByDeviceId {}", request.device_id);
                    printers::err(msg);
                }
            });
        },
        ValueRequest::GetAll(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_all_values(&pool).await;
                if tx.send(res).is_err() {
                    printers::err(String::from("Помилка відправки ValueRequest::GetAll"));
                }
            });
        },
        ValueRequest::GetByGroup(request) => {
            unimplemented!();
        },
        ValueRequest::GetLoggingOnly(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_logging_only(&pool).await;
                if tx.send(res).is_err() {
                    printers::err(String::from("Помилка відправки ValueRequest::GetLoggingOnly"));
                }
            });
        }
    }
}

async fn get_value_by_id(pool: &Pool<MySql>, id: i32) -> Result<Option<ValueRead>, ()> {
    let value = sqlx::query_as::<_, ValueRead>(
        "SELECT id,
       parent_device_id,
       value_name,
       value_tag,
       description,
       decoding_type,
       settings,
       is_logging
        FROM value_units WHERE id = ?"
    )
        .bind(id)
        .fetch_optional(pool)
        .await;
    match value {
        Ok(value) => {
            if value.is_none() {
                Ok(None)
            } else {
                Ok(value)
            }
        }
        Err(e) => {
            let msg = format!("Помилка отримання значення від БД по id: {:?}", e);
            printers::err(msg);
            Err(())
        },
    }
}

async fn get_value_by_device_id(pool: &Pool<MySql>, id: i32) -> Result<Vec<ValueRead>, ()> {
    let values = sqlx::query_as::<_, ValueRead>(
        "SELECT id,
       parent_device_id,
       value_name,
       value_tag,
       description,
       decoding_type,
       settings,
       is_logging
        FROM value_units WHERE parent_device_id = ?"
    )
        .bind(id)
        .fetch_all(pool)
        .await;
    match values {
        Ok(values) => Ok(values),
        Err(e) => {
            let msg = format!("Помилка отримання значення від БД по device_id: {:?}", e);
            printers::err(msg);
            Err(())
        }
    }
}

async fn get_all_values(pool: &Pool<MySql>) -> Result<Vec<ValueRead>, ()> {
    let values = sqlx::query_as::<_, ValueRead>(
        "SELECT id,
       parent_device_id,
       value_name,
       value_tag,
       description,
       decoding_type,
       settings,
       is_logging
        FROM value_units"
    )
        .fetch_all(pool)
        .await;
    match values {
        Ok(values) => Ok(values),
        Err(e) => {
            let msg = format!("Помилка отримання усих значень від БД: {:?}", e);
            printers::err(msg);
            Err(())
        }
    }
}

async fn get_logging_only(pool: &Pool<MySql>) -> Result<Vec<ValueRead>, ()> {
    let values = sqlx::query_as::<_, ValueRead>(
        "SELECT id,
       parent_device_id,
       value_name,
       value_tag,
       description,
       decoding_type,
       settings,
       is_logging
        FROM value_units WHERE is_logging = 1"
    )
        .fetch_all(pool)
        .await;
    match values {
        Ok(values) => Ok(values),
        Err(e) => {
            let msg = format!("Помилка отримання усих значень з логуванням від БД: {:?}", e);
            printers::err(msg);
            Err(())
        }
    }
}

async fn delete_value(pool: &Pool<MySql>, value: &ValueDelete) -> Result<(), String> {
    let res = sqlx::query("DELETE FROM value_units WHERE id = ?")
        .bind(value.id)
        .execute(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка видалення значення id: {} \n{}", value.id, &e);
            printers::err(msg.clone());
            msg
        })?;

    let msg = format!("Видалено значення : {:?}", value);
    printers::event(msg);

    Ok(())
}

async fn create_value(pool: &Pool<MySql>, value: &ValueCreate) -> Result<(), String> {

    let dev = sqlx::query("SELECT id FROM devices WHERE id = ? AND deleted = 0")
        .bind(value.parent_device_id)
        .fetch_optional(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка читання з бази даних при перевірці пристрою: \n{}", e);
            printers::err(msg.clone());
            msg
        })?;
    if dev.is_none() {
        let msg = format!("Відсутній пристрій (device) з таким id: {}", value.parent_device_id);
        printers::err(msg.clone());
        return Err(msg)
    };

    sqlx::query(
        "INSERT INTO value_units (parent_device_id, value_name, value_tag, description, decoding_type, settings, is_logging) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(value.parent_device_id)
        .bind(&value.value_name)
        .bind(&value.value_tag)
        .bind(&value.description)
        .bind(value.decoding_type)
        .bind(&value.settings)
        .bind(value.is_logging)
        .execute(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка запису в базу даних:\n{}", e);
            printers::err(msg.clone());
            msg
        })?;

    let msg = format!("Створено значення : {:?}", value);
    printers::event(msg);

    Ok(())
}

async fn value_update(pool: &Pool<MySql>, value: &ValueUpdate) -> Result<(), String> {

    let val = sqlx::query("SELECT id FROM value_units WHERE id = ?")
        .bind(value.id)
        .fetch_optional(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка читання з бази даних: \n{}", e);
            printers::err(msg.clone());
            msg
        })?;
    if val.is_none() {
        let msg = format!("Запис (Value) не знайдено, id: {}", value.id);
        printers::err(msg.clone());
        return Err(msg)
    }
    if let Some(parent_id) = value.parent_device_id {  // TODO розширити перевірку на унікальність адрес та ID значень для конкретного пристрою
        let dev = sqlx::query("SELECT id FROM devices WHERE id = ? AND deleted = 0")
            .bind(parent_id)
            .fetch_optional(pool)
            .await
            .map_err(|e|{
                let msg = format!("Помилка читання з бази даних при валідації девайса: \n{}", e);
                printers::err(msg.clone());
                msg
            })?;
        if dev.is_none() {
            let msg = format!("Вказаний Device не існує або видалений: \n{}", parent_id);
            printers::err(msg.clone());
            return Err(msg)
        }
    }
    sqlx::query(
        r#"
        UPDATE value_units SET
            parent_device_id = COALESCE(?, parent_device_id),
            value_name = COALESCE(?, value_name),
            value_tag = COALESCE(?, value_tag),
            description = COALESCE(?, description),
            decoding_type = COALESCE(?, decoding_type),
            settings = COALESCE(?, settings),
            is_logging = COALESCE(?, is_logging)
        WHERE id = ?
        "#
    )
        .bind(value.parent_device_id)
        .bind(&value.value_name)
        .bind(&value.value_tag)
        .bind(&value.description)
        .bind(&value.decoding_type)
        .bind(&value.settings) // Прямий біндінг Option<Value>, sqlx розбереться самостійно
        .bind(value.is_logging)
        .bind(value.id)
        .execute(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка оновлення бази даних (value_units): \n{}", e);
            printers::err(msg.clone());
            msg
        })?;


    let msg = format!("Оновлено значення : {:?}", value);
    printers::event(msg);

    Ok(())
}