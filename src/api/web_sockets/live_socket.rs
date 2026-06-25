use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use crate::logger::printers;
use std::time::Duration;
use tokio::time::{sleep, timeout};



use crate::api::init_axum::AppState;
use tokio::sync::mpsc;
use crate::api::web_sockets::live_socket_unit::CoordUnitWebSocketData;


pub fn live_router() -> Router<AppState> {
    Router::new()
        .route("/live_data", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let (_, mut receiver) = mpsc::channel(10);

    let duration = Duration::from_secs(30);

    match timeout(duration, socket.recv()).await {
        Ok(read_some) => {
            match read_some {
                Some(Ok(Message::Text(config))) => {
                    if let Ok(rec) = new_config(&mut socket, &state.to_ws_coord, config).await {
                        receiver = rec;
                    } else {
                        printers::warn("Помилка повідомлення вебсокету".to_string());
                        return
                    }
                },
                _ => {
                    printers::warn("Падіння вебсокету".to_string());
                    return
                },
            }
        }
        Err(_) => {
            printers::warn("Таймаут по підключенні вебсокету".to_string());
            return;
        }
    }

    printers::event("Отримано нове вебсокет підключення".to_string());

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(config))) => {
                        if let Ok(rec) = new_config(&mut socket, &state.to_ws_coord, config).await {
                            receiver = rec;
                        } else {
                            printers::warn("Помилка повідомлення вебсокету".to_string());
                            break;
                        }
                    },
                    _ => {
                        printers::event("Сокет закрив з'єднання".to_string());
                        break;
                    },
                    }
            }
            event = receiver.recv() => {
                match event {
                    Some(message) => {
                        if let Err(e) = socket.send(Message::Text(message)).await {
                            printers::err(format!("Помилка відправки подій в вебсокет: {:?}", e));
                            break;
                        }
                    },
                    None => break, // канал упав
                }
            }
        }
    }
}

async fn new_config(socket: &mut WebSocket, to_ws_coord: &mpsc::Sender<CoordUnitWebSocketData>, config: String) -> Result<mpsc::Receiver<String>, ()> {
    let (sender, receiver) = mpsc::channel(10);

    let send_unit = CoordUnitWebSocketData::new(config.as_str(), sender).map_err(|e| {
        printers::warn(format!("Невалідна конфігурація вебсокету: {:?}", e));
    })?;

    match to_ws_coord.send(send_unit).await {
        Ok(_) => Ok(receiver),
        Err(e) => {
            let msg = format!("Помилка відправки конфігурації вебсокету: {:?}", e);
            printers::err(msg.clone());
            if let Err(e) = socket.send(Message::Text(msg)).await {
                printers::err(format!("Помилка відправки зворотнього звязку вебсокету: {:?}", e));
            }
            Err(())
        }
    }
}