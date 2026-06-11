use axum::{extract::State, Json, extract::Path, Router};
use axum::routing::{get, post, put, delete};
use crate::db::schemas::value_unit::*;
use crate::db::states::AppState;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use crate::logger::printers;

pub fn values_router() -> Router<AppState> {
    Router::new()
        .route("/values/get_all", get(get_all_values))
        .route("/values/create", post(create_value))
        .route("/values/get_logging_only", get(get_logging_values))
        .route("/values/get_by_parent_id/:id", get(get_by_parent_id))
        .route("/values/update/:id", put(update_value))
        .route("/values/delete/:id", delete(delete_value))
}

pub async fn get_all_values(State(state): State<AppState>) -> impl IntoResponse {
    // Назви полів адаптовано під нову таблицю value_units
    let values = sqlx::query_as::<_, ValueRead>(
        "SELECT id, parent_device_id, value_name, value_tag, description, decoding_type, settings, is_logging FROM value_units"
    )
        .fetch_all(&state.pool)
        .await;

    match values {
        Ok(values) => (StatusCode::OK, Json(values)).into_response(),
        Err(e) => {
            let error = format!("Помилка читання значень:\n {}", e);
            printers::err(error.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
        }
    }
}

pub async fn create_value(State(state): State<AppState>, Json(payload): Json<ValueCreate>) -> impl IntoResponse {
    // Перевірка валідності назви тегу (бо це UNIQUE бізнес-ключ)
    if payload.value_tag.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Тег значення (value_tag) не може бути пустим").into_response();
    }

    // Перевірка існування батьківського девайсу
    let dev = sqlx::query("SELECT id FROM devices WHERE id = ? AND deleted = 0")
        .bind(payload.parent_device_id)
        .fetch_optional(&state.pool)
        .await;

    match dev {
        Ok(Some(_)) => {},
        Ok(None) => {
            return (StatusCode::BAD_REQUEST, "Відсутній пристрій (device) з таким id").into_response();
        },
        Err(e) => {
            let msg = format!("Помилка читання з бази даних при перевірці пристрою: \n{}", e);
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
        }
    }

    // Запис у нову таблицю з урахуванням усіх полів
    let res = sqlx::query(
        "INSERT INTO value_units (parent_device_id, value_name, value_tag, description, decoding_type, settings, is_logging) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(payload.parent_device_id)
        .bind(payload.value_name)
        .bind(payload.value_tag)
        .bind(payload.description)
        .bind(payload.decoding_type)
        .bind(payload.settings) // sqlx сам конвертує serde_json::Value в MySQL JSON
        .bind(payload.is_logging)
        .execute(&state.pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка запису в базу даних:\n{}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn get_logging_values(State(state): State<AppState>) -> impl IntoResponse {
    // Більше ніякого ручного перебору в циклі! База даних сама робить вибірку по індексу/полю.
    let res = sqlx::query_as::<_, ValueRead>(
        "SELECT id, parent_device_id, value_name, value_tag, description, decoding_type, settings, is_logging
         FROM value_units WHERE is_logging = 1"
    )
        .fetch_all(&state.pool)
        .await;

    match res {
        Ok(values) => (StatusCode::OK, Json(values)).into_response(),
        Err(e) => {
            let msg = format!("Помилка читання з бази даних (h_logging): \n{}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn get_by_parent_id(State(state): State<AppState>, Path(id): Path<u32>) -> impl IntoResponse {
    let res = sqlx::query_as::<_, ValueRead>(
        "SELECT id, parent_device_id, value_name, value_tag, description, decoding_type, settings, is_logging
         FROM value_units WHERE parent_device_id = ?"
    )
        .bind(id)
        .fetch_all(&state.pool)
        .await;

    match res {
        Ok(values) => (StatusCode::OK, Json(values)).into_response(),
        Err(e) => {
            let msg = format!("Помилка читання з бази даних: \n{}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn update_value(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(payload): Json<ValueUpdate>,
) -> impl IntoResponse {

    // 🔹 Перевірка існування поточного запису value_unit
    let val = sqlx::query("SELECT id FROM value_units WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    match val {
        Ok(Some(_)) => {},
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Запис (Value) не знайдено").into_response();
        },
        Err(e) => {
            let msg = format!("Помилка читання з бази даних: \n{}", e);
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }

    // 🔹 Перевірка device (тільки якщо передали для оновлення)
    if let Some(parent_id) = payload.parent_device_id {
        let dev = sqlx::query("SELECT id FROM devices WHERE id = ? AND deleted = 0")
            .bind(parent_id)
            .fetch_optional(&state.pool)
            .await;

        match dev {
            Ok(Some(_)) => {},
            Ok(None) => {
                return (StatusCode::BAD_REQUEST, "Вказаний Device не існує або видалений").into_response();
            },
            Err(e) => {
                let msg = format!("Помилка читання з бази даних при валідації девайса: \n{}", e);
                printers::err(msg.clone());
                return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }

    // 🔹 Оновлення через COALESCE. Передаємо payload.settings безпосередньо.
    let res = sqlx::query(
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
        .bind(payload.parent_device_id)
        .bind(payload.value_name)
        .bind(payload.value_tag)
        .bind(payload.description)
        .bind(payload.decoding_type)
        .bind(payload.settings) // Прямий біндінг Option<Value>, sqlx розбереться самостійно
        .bind(payload.is_logging)
        .bind(id)
        .execute(&state.pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка оновлення бази даних (value_units): \n{}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn delete_value(State(state): State<AppState>, Path(id): Path<u32>) -> impl IntoResponse {
    // Оскільки в DDL для value_units немає поля soft delete (deleted), робимо класичний DELETE.
    let res = sqlx::query("DELETE FROM value_units WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка видалення значення з бази даних:\n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}