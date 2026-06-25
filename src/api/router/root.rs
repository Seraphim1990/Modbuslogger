/*use crate::db::states::AppState;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

pub fn root() -> Router<AppState> {
    Router::new().route("/", get(index))
}

pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    Html((*state.index_html).clone())
}

 */