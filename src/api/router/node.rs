use axum::{extract::State, Json, extract::Path, Router};
use axum::routing::{get, post, put, delete};
use crate::db::schemas::node::*;
use crate::db::states::AppState;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use crate::logger::printers;

use std::net::IpAddr;


pub fn node_router() -> Router<AppState> {
    Router::new()
        .route("/nodes/get_all", get(get_nodes))
        .route("/nodes/:id", get(get_node_by_id))
        .route("/nodes/create", post(create_node))
        .route("/nodes/update/:id", put(update_node))
        .route("/nodes/delete/:id", delete(delete_node))
        .route("/nodes/get_node_by_ip/:ip", get(get_node_by_ip))
}

pub async fn get_nodes(State(state): State<AppState>) -> impl IntoResponse {
    let nodes = sqlx::query_as::<_, NodeRead>("SELECT id, ip, port, description FROM nodes")
        .fetch_all(&state.pool)
        .await;

    match nodes {
        Ok(nodes) => (StatusCode::OK, Json(nodes)).into_response(),
        Err(e) => {
            let msg = format!("Помилка читання бази данних, fn get_nodes: \n{}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn get_node_by_ip(State(state): State<AppState>, Path(ip): Path<String>) -> impl IntoResponse {
    let node = sqlx::query_as::<_, NodeRead>(
        "SELECT id, ip, port, description FROM nodes WHERE ip = ?"
    )
        .bind(ip)
        .fetch_optional(&state.pool)
        .await;

    match node {
        Ok(Some(node)) => (StatusCode::OK, Json(node)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(())).into_response(),
        Err(e) => {
            let msg = format!("Помилка читання бази данних, fn get_node_by_id, \n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn get_node_by_id(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    let node = sqlx::query_as::<_, NodeRead>(
        "SELECT id, ip, port, description FROM nodes WHERE id = ?"
    )
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    match node {
        Ok(Some(node)) => (StatusCode::OK, Json(node)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Ноду не знайдено").into_response(),
        Err(e) => {
            let msg = format!("Помилка читання бази данних, fn get_node_by_id, \n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn create_node(State(state): State<AppState>, Json(payload): Json<NodeCreate>) -> impl IntoResponse {
    
    let ip = payload.ip.clone();
    if ip.parse::<IpAddr>().is_err() {
        let msg = format!("Невірний ip: {}", ip);
        printers::err(msg.clone());
        return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
    }

    let node = sqlx::query_as::<_, NodeRead>(
        "SELECT id, ip, port, description FROM nodes WHERE ip = ?"
    )
        .bind(ip.clone())
        .fetch_optional(&state.pool)
        .await;

    match node {
        Ok(Some(_)) => {
            let msg = format!("Помилка створення ноди:{}, нода з таким ip уже існує", ip.clone());
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        },
        Ok(None) => {},
        Err(e) => {
            let msg = format!("Помилка створення ноди:\n {}", e);
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }

    let port = if let Some(port) = payload.port {port} else {502};

        if port < 0 || port > 65535 {
            let msg = format!("Помилка створення ноди:{}, не валідний порт", port);
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }

    let res = sqlx::query("INSERT INTO nodes (ip, port, description) VALUES (?, ?, ?)")
        .bind(payload.ip)
        .bind(port)
        .bind(payload.description)
        .execute(&state.pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка створення ноди:\n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn update_node(State(state): State<AppState>,
                         Path(id): Path<u32>,
                         Json(payload): Json<NodeUpdate>) -> impl IntoResponse
{
    let node = sqlx::query_as::<_, NodeRead>(
        "SELECT id, ip, port, description FROM nodes WHERE id = ?"
    )
        .bind(id)
        .fetch_optional(&state.pool)
        .await;
    match node {
        Ok(Some(_)) => {},
        Ok(None) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Помилка знаходження ноди: {}", id)).into_response(),
        Err(e) => {
            let msg = format!("Помилка бази даних:\n {}", e);
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }

    if let Some(port) = payload.port {
        if port < 0 || port > 65535 {
            let msg = format!("Помилка створення ноди:{}, не валідний порт", payload.port.unwrap());
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }

    if payload.ip.is_some() {
        let ip = payload.ip.clone().unwrap();
        if ip.parse::<IpAddr>().is_err() {
            let msg = format!("Невірний ip: {}", ip);
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }

        let node = sqlx::query_as::<_, NodeRead>(
            "SELECT id, ip, port, description FROM nodes WHERE ip = ?"
        )
            .bind(ip.clone())
            .fetch_optional(&state.pool)
            .await;

        match node {
            Ok(Some(node)) => {
                if node.id != id as i32 {
                    let msg = format!("ip належить іншій ноді: {}", ip);
                    printers::err(msg.clone());
                    return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
                }
            },
            Ok(None) => {},
            Err(e) => {
                let msg = format!("Помилка читання з бази данних: {}", e);
                printers::err(msg.clone());
                return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }

    let res = sqlx::query(
        "UPDATE nodes SET ip = COALESCE(?, ip), port = COALESCE(?, port), description = COALESCE(?, description) WHERE id = ?"
        )
        .bind(payload.ip)
        .bind(payload.port)
        .bind(payload.description)
        .bind(id)
        .execute(&state.pool)
        .await;

    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка оновлення ноди:\n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn delete_node(State(state): State<AppState>, Path(id): Path<u32>,) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM nodes WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await;
    match res {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            let msg = format!("Помилка видалення ноди:\n {}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}
