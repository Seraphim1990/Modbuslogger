use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use crate::logger::printers;
use std::time::Duration;
use axum::extract::Query;
use axum::http::StatusCode;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use tokio::time::{sleep, timeout};



use crate::api::init_axum::{AppState, Claims, JWT_KEY};
use tokio::sync::mpsc;
use crate::api::web_sockets::live_socket_unit::CoordUnitWebSocketData;


pub fn live_router() -> Router<AppState> {
    Router::new()
        .route("/live_data", get(ws_handler))
}

#[derive(Deserialize)]
pub struct WsParams {
    token: Option<String>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>, // Axum автоматично дістає ?token= із запиту
    State(state): State<AppState>,
) -> impl IntoResponse {

    if let Some(token_str) = params.token {
        if let Ok(_) = decode::<Claims>(
            token_str,
            &DecodingKey::from_secret(JWT_KEY),
            &Validation::default(), // Перевіряє exp і iat автоматично
        ){
            return ws.on_upgrade(move |socket| handle_socket(socket, state));
        }
    }
    StatusCode::UNAUTHORIZED.into_response()
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut receiver;

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

    let timer = sleep(Duration::from_secs(900));
    tokio::pin!(timer);
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
            _ = &mut timer => { // розрив зьєднання для отримання нового токену і перепідключення
                break;
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