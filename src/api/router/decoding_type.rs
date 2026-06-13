use axum::{extract::State, Json, extract::Path, Router};
use axum::routing::{get};
use crate::db::schemas::decoding_type::*;
use crate::db::states::AppState;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use crate::logger::printers;



pub fn decoding_type() -> Router<AppState> {
    Router::new()
        .route("/decoding_type/get_all", get(get_all_decoders))
        .route("/decoding_type/get_addons/:id", get(get_qml_addon))
}

pub async fn get_all_decoders(State(state): State<AppState>) -> impl IntoResponse {
    let res = sqlx::query_as::<_, DecodingType>("SELECT id, uiName, programName FROM decodingtype")
        .fetch_all(&state.pool)
        .await;
    match res {
        Ok(decoded) => (StatusCode::OK, Json(decoded)).into_response(),
        Err(e) => {
            let msg = format!("Помилка читання з бази даних: \n{}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn get_qml_addon(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    // 🔹 спробуємо знайти addon
    let res = sqlx::query_as::<_, QmlDecodingAddons>(
        "SELECT id, parentDescriptionType, textForQML
         FROM qmldecodingaddons
         WHERE parentDescriptionType = ?"
    )
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    let addon = match res {
        Ok(Some(addon)) => addon, // знайдено, повертаємо
        Ok(None) => {
            // якщо не знайдено, перевіряємо DecodingType
            let dt_res = sqlx::query_as::<_, DecodingType>(
                "SELECT id, uiName FROM decoding_types WHERE id = ?"
            )
                .bind(id)
                .fetch_optional(&state.pool)
                .await;

            match dt_res {
                Ok(Some(dt)) => {
                    // DecodingType є, але addon немає
                    let msg = format!("No addons found for decoding type {}", dt.ui_name);
                    return (StatusCode::NOT_FOUND, msg).into_response();
                },
                Ok(None) => {
                    // DecodingType теж немає
                    let msg = format!("Decoding type with id {} not found", id);
                    return (StatusCode::NOT_FOUND, msg).into_response();
                },
                Err(e) => {
                    let msg = format!("DB error: {}", e);
                    printers::err(msg.clone());
                    return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
                }
            }
        },
        Err(e) => {
            let msg = format!("DB error: {}", e);
            printers::err(msg.clone());
            return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
        }
    };

    // Якщо все ок — повертаємо знайдений addon
    (StatusCode::OK, Json(addon)).into_response()
}