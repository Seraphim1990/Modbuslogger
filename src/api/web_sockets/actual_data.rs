use std::sync::atomic::{AtomicU64, Ordering};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Serialize};
use crate::api::web_sockets::socket_types::*;
use crate::db::states::AppState;
use crate::logger::printers::event;

static CLIENT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);


// broadcast-канал для розсилки оновлень між клієнтами
#[derive(Clone)]
pub struct BroadcastUpdate {
    pub sender_id: u64,
    pub updated_values: Vec<HashedValue>,
}


// --- Роут ---

pub fn actual_data_router() -> Router<AppState> {
    Router::new()
        .route("/actual_data", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// --- Обробка WebSocket ---

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let client_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut subscribed_tags: Vec<String> = vec![];
    let mut rx = state.tx.subscribe();

    event(String::from("Підключення нового клієнта до веб сокету"));

    loop {
        tokio::select! {
            // Повідомлення від цього клієнта
            msg = socket.recv() => {
                let msg = match msg {
                    Some(Ok(Message::Text(text))) => text,
                    // Клієнт від'єднався або помилка
                    _ => break,
                };

                let parsed: ClientMessage = match serde_json::from_str(&msg) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                match parsed {
                    ClientMessage::Update { update } => {
                        state.vault.update_values(update.clone());

                        let _ = socket.send(Message::Text(
                            "Values updated successfully".into()
                        )).await;

                        // Розсилаємо всім іншим через broadcast
                        let _ = state.tx.send(BroadcastUpdate {
                            sender_id: client_id,
                            updated_values: update,
                        });
                    }

                    ClientMessage::Get { get } => {
                        subscribed_tags = get.clone();
                        let values = state.vault.get_many(&get);
                        send_json(&mut socket, &values).await;
                    }

                    ClientMessage::GetAll { .. } => {
                        subscribed_tags.clear();
                        let values = state.vault.get_all();
                        send_json(&mut socket, &values).await;
                    }
                }
            }

            // Оновлення від інших клієнтів
            Ok(update) = rx.recv() => {
                if update.sender_id == client_id {  // ← фільтр
                    continue;
                }

                if subscribed_tags.is_empty() {
                    continue;
                }

                let filtered: Vec<&HashedValue> = update
                    .updated_values
                    .iter()
                    .filter(|v| subscribed_tags.contains(&v.tag))
                    .collect();

                if !filtered.is_empty() {
                    send_json(&mut socket, &filtered).await;
                }
            }
        }
    }
    event(String::from("Відключення клієнта від веб сокету"));
}

async fn send_json<T: Serialize>(socket: &mut WebSocket, data: &T) {
    if let Ok(json) = serde_json::to_string(data) {
        let _ = socket.send(Message::Text(json.into())).await;
    }
}