use axum::response::{IntoResponse, Response};
use axum::body::Body;
use tokio::fs;
use axum::{routing::get, Router};
use crate::api::init_axum::AppState;

pub fn root() -> Router<AppState> {
    Router::new()
        .route("/", get(serve_html))
        .route("/static/admin.css", get(serve_css))
        .route("/static/admin.js", get(serve_js))
        .route("/static/schemas.js", get(serve_schemas))
}


async fn serve_html() -> impl IntoResponse {
    let content = tokio::fs::read_to_string("static/admin.html").await.unwrap_or_default();
    Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(content))
        .unwrap()
}

async fn serve_css() -> impl IntoResponse {
    let content = fs::read_to_string("static/admin.css").await.unwrap_or_default();
    Response::builder()
        .header("Content-Type", "text/css")
        .body(Body::from(content))
        .unwrap()
}

async fn serve_js() -> impl IntoResponse {
    let content = fs::read_to_string("static/admin.js").await.unwrap_or_default();
    Response::builder()
        .header("Content-Type", "application/javascript")
        .body(Body::from(content))
        .unwrap()
}

async fn serve_schemas() -> impl IntoResponse {
    let content = fs::read_to_string("static/schemas.js").await.unwrap_or_default();
    Response::builder()
        .header("Content-Type", "application/javascript")
        .body(Body::from(content))
        .unwrap()
}