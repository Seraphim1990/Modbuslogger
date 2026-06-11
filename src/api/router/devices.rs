use axum::{extract::State, Json, extract::Path, Router};
use axum::extract::Query;
use axum::routing::{get, post, put, delete};
use crate::db::schemas::device::*;
use crate::db::states::AppState;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use serde::Deserialize;
use crate::db::schemas::node::{NodeRead};
use crate::logger::printers;

pub fn devices_router() -> Router<AppState> {
    Router::new()
        .route("/devices/get_all", get(get_devices))
        .route("/devices/get_by_parent_id/:id", get(get_devices_by_parent_id))
        .route("/devices/find_exists_addr_wit_parent_id", get(find_exists_addr_wit_parent_id))
        .route("/devices/get_device/:id", get(get_device))
        .route("/devices/create", post(create_devices))
        .route("/devices/update/:id", put(update_device))
        .route("/devices/delete/:id", delete(delete_device))
}

pub async fn create_devices(State(state): State<AppState>, Json(payload): Json<DeviceCreate>) -> impl IntoResponse {
    // Валідація вхідних даних
    if payload.address < 0 || payload.address > 255 {
        return (StatusCode::BAD_REQUEST, "Не вірна адреса пристрою").into_response();
    }
    if payload.time_for_recall <= 0 {
        return (StatusCode::BAD_REQUEST, "Час опитування не може бути від'ємним або нулем").into_response();
    }
    if payload.retry_count <= 0 {
        return (StatusCode::BAD_REQUEST, "Кількість повторів не може бути від'ємною або нулем").into_response();
    }
    if payload.timeout <= 0 {
        return (StatusCode::BAD_REQUEST, "Час таймауту (timeout) не може бути від'ємним або нулем").into_response();
    }
    if payload.parent_node_id <= 0 {
        return (StatusCode::BAD_REQUEST, "id Батьківської ноди не може бути від'ємним").into_response();
    }

    // Перевірка існування батьківської ноди
    let node = sqlx::query_as::<_, NodeRead>(
        "SELECT id, ip, port, description FROM nodes WHERE id = ?"
    )
        .bind(payload.parent_node_id)
        .fetch_optional(&state.pool)
        .await;

    match node {
        Ok(Some(_)) => {},
        Ok(None) => {
            return (StatusCode::BAD_REQUEST, "Відсутня нода з таким id").into_response();
        },
        Err(e) => {
            let msg = format!("Помилка читання бази даних, fn get_node_by_id, \n {}", e);
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }

    // Запит адаптовано під новий DDL (без time_for_retry / response, додано timeout)
    let res = sqlx::query(
        "INSERT INTO devices (parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(payload.parent_node_id)
        .bind(payload.device_name)
        .bind(payload.address)
        .bind(payload.time_for_recall)
        .bind(payload.timeout)
        .bind(payload.retry_count)
        .bind(payload.is_active)
        .bind(payload.read_by_group)
        .bind(payload.description)
        .execute(&state.pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка запису в базу даних, \n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn get_devices(State(state): State<AppState>) -> impl IntoResponse {
    // Вибираємо тільки не видалені пристрої (додано фільтр WHERE deleted = 0 або deleted IS NULL)
    let devices = sqlx::query_as::<_, DeviceRead>(
        "SELECT id, parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description, deleted, deleted_at FROM devices WHERE deleted = 0",
    ).fetch_all(&state.pool).await;

    match devices {
        Ok(devices) => (StatusCode::OK, Json(devices)).into_response(),
        Err(e) => {
            let msg = format!("Помилка читання бази даних, fn get_devices: \n{}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn get_devices_by_parent_id(
    State(state): State<AppState>,
    Path(parent_id): Path<u32>,
) -> impl IntoResponse {
    let devices = sqlx::query_as::<_, DeviceRead>(
        "SELECT id, parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description, deleted, deleted_at
        FROM devices WHERE parent_node_id = ? AND deleted = 0"
    )
        .bind(parent_id)
        .fetch_all(&state.pool)
        .await;

    match devices {
        Ok(devices) => {
            if devices.is_empty() {
                let msg = format!("No devices found for node {}", parent_id);
                return (StatusCode::NOT_FOUND, msg).into_response();
            }
            (StatusCode::OK, Json(devices)).into_response()
        },
        Err(e) => {
            let msg = format!("Помилка читання бази даних: {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct DeviceQuery {
    parent_id: u32,
    address: Option<u32>,
}

pub async fn find_exists_addr_wit_parent_id(State(state): State<AppState>, Query(params): Query<DeviceQuery>) -> impl IntoResponse {
    let device = sqlx::query_as::<_, DeviceRead>(
        "SELECT id, parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description, deleted, deleted_at
         FROM devices
         WHERE address = ? AND parent_node_id = ? AND deleted = 0"
    )
        .bind(params.address)
        .bind(params.parent_id)
        .fetch_optional(&state.pool)
        .await;

    match device {
        Ok(Some(device)) => (StatusCode::OK, Json(device)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Device not found").into_response(),
        Err(e) => {
            let msg = format!("Помилка читання бази даних: {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn get_device(State(state): State<AppState>, Path(id): Path<u32>) -> impl IntoResponse {
    let device = sqlx::query_as::<_, DeviceRead>(
        "SELECT id, parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description, deleted, deleted_at
        FROM devices WHERE id = ? AND deleted = 0",
    )
        .bind(id).fetch_optional(&state.pool).await;
    match device {
        Ok(Some(device)) => (StatusCode::OK, Json(device)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Пристрій не знайдено").into_response(),
        Err(e) => {
            let msg = format!("Помилка читання бази даних: {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn update_device(State(state): State<AppState>, Path(id): Path<u32>, Json(payload): Json<DeviceUpdate>) -> impl IntoResponse {
    // Валідація Option-полів безпечним способом
    if let Some(addr) = payload.address {
        if addr < 0 || addr > 255 {
            return (StatusCode::BAD_REQUEST, "Не вірна адреса пристрою").into_response();
        }
    }
    if let Some(recall) = payload.time_for_recall {
        if recall < 0 {
            return (StatusCode::BAD_REQUEST, "Час опитування не може бути від'ємним").into_response();
        }
    }
    if let Some(timeout) = payload.timeout {
        if timeout < 0 {
            return (StatusCode::BAD_REQUEST, "Час таймауту не може бути від'ємним").into_response();
        }
    }
    if let Some(retry) = payload.retry_count {
        if retry < 0 {
            return (StatusCode::BAD_REQUEST, "Кількість повторів не може бути від'ємною").into_response();
        }
    }

    // Якщо прилітає parent_node_id, перевіряємо його на валідність та існування ноди
    if let Some(p_node_id) = payload.parent_node_id {
        if p_node_id <= 0 {
            return (StatusCode::BAD_REQUEST, "id Батьківської ноди не може бути від'ємним").into_response();
        }

        let node = sqlx::query_as::<_, NodeRead>(
            "SELECT id, ip, port, description FROM nodes WHERE id = ?"
        )
            .bind(p_node_id)
            .fetch_optional(&state.pool)
            .await;

        match node {
            Ok(Some(_)) => {},
            Ok(None) => {
                return (StatusCode::BAD_REQUEST, "Ноди з таким id не існує").into_response();
            },
            Err(e) => {
                let msg = format!("Помилка бази даних при перевірці ноди: \n{}", e);
                printers::err(msg.clone());
                return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }

    if payload.address.is_some() || payload.parent_node_id.is_some() {
        let device = sqlx::query_as::<_, DeviceRead>(
            "SELECT id, parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description, deleted, deleted_at
             FROM devices
             WHERE address = COALESCE(?, address) AND parent_node_id = COALESCE(?, parent_node_id) AND deleted = 0"
        )
            .bind(payload.address)
            .bind(payload.parent_node_id)
            .fetch_optional(&state.pool)
            .await;

        match device {
            Ok(Some(dev)) => {
                if dev.id != id as i32 {
                    return (StatusCode::BAD_REQUEST, "Ця нода уже має пристрій з такою адресою").into_response();
                }
            },
            Ok(_) => {},
            Err(e) => {
                let msg = format!("Помилка перевірки унікальності адреси: {}", e);
                printers::err(msg.clone());
                return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            },
        }
    }
    let res = sqlx::query(
        "UPDATE devices
         SET
            device_name = COALESCE(?, device_name),
            address = COALESCE(?, address),
            parent_node_id = COALESCE(?, parent_node_id),
            time_for_recall = COALESCE(?, time_for_recall),
            timeout = COALESCE(?, timeout),
            retry_count = COALESCE(?, retry_count),
            is_active = COALESCE(?, is_active),
            read_by_group = COALESCE(?, read_by_group),
            description = COALESCE(?, description)
         WHERE id = ?"
    )
        .bind(payload.device_name)
        .bind(payload.address)
        .bind(payload.parent_node_id)
        .bind(payload.time_for_recall)
        .bind(payload.timeout)
        .bind(payload.retry_count)
        .bind(payload.is_active)
        .bind(payload.read_by_group)
        .bind(payload.description)
        .bind(id)
        .execute(&state.pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка оновлення пристрою:\n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

// Замість повного DELETE робимо Soft Delete (безпечне видалення), як закладено в DDL
pub async fn delete_device(State(state): State<AppState>, Path(id): Path<u32>) -> impl IntoResponse {
    let current_timestamp = chrono::Utc::now().timestamp();

    let res = sqlx::query("UPDATE devices SET deleted = 1, deleted_at = ? WHERE id = ?")
        .bind(current_timestamp)
        .bind(id)
        .execute(&state.pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка видалення пристрою:\n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}