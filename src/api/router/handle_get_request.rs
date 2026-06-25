use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tokio::sync::{mpsc, oneshot};
use crate::logger::printers;
use crate::messages::main_msg::MainMsg;

pub async fn handle_get_request<T, B>(
    response: oneshot::Receiver<Result<T, B>>,
    send_chanel: mpsc::Sender<MainMsg>,
    msg: MainMsg
) -> Response
where T: Serialize
{
    if let Err(e) = check_send_message(&send_chanel, msg).await{
        return e.into_response();
    };

    match response.await {
        Ok(Ok(res)) => {(StatusCode::OK, Json(res)).into_response()},
        Ok(Err(_)) => {
            let msg = "Помилка читання бази данних, детальніше в логах".to_string();
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        },
        Err(e) => {
            let msg = format!("Помилка каналу: \n{}", e);
            printers::err(msg.clone());
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

pub async fn check_send_message(send_chanel: &mpsc::Sender<MainMsg>, message: MainMsg) -> Result<(), impl IntoResponse> {
    if let Err(e) = send_chanel.send(message).await {
        let msg = format!("Помилка сервера: {}", e);
        printers::err(msg.clone());
        return Err((StatusCode::INTERNAL_SERVER_ERROR, msg).into_response());
    };
    Ok(())
}