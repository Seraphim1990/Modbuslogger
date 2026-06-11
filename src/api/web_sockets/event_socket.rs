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
use std::collections::HashMap;
use std::sync::Mutex;
use crate::db::states::AppState;
use crate::logger::printers::event;
use std::sync::atomic::{AtomicU64, Ordering};

static CLIENT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EventClientMessage {
    Update { update: HashMap<String, serde_json::Value> },
    Get { get: Vec<String> },
}

#[derive(Clone)]
pub struct EventBroadcastUpdate {
    pub sender_id: u64,
    pub updated_events: HashMap<String, serde_json::Value>,
}

// --- EventDataContainer ---

#[derive(Default)]
pub struct EventDataContainer {
    events: Mutex<HashMap<String, serde_json::Value>>,
}

impl EventDataContainer {
    pub fn update_event(&self, updates: HashMap<String, serde_json::Value>) {
        let mut store = self.events.lock().unwrap();
        for (tag, value) in updates {
            store.insert(tag, value);
        }
    }

    pub fn get_event(&self, tag: &str) -> Option<serde_json::Value> {
        self.events.lock().unwrap().get(tag).cloned()
    }
}

// --- Роут ---

pub fn events_router() -> Router<AppState> {
    Router::new()
        .route("/events", get(ws_handler))
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
    let mut rx = state.event_tx.subscribe();

    event(String::from("Підключення нового клієнта до events сокету"));

    loop {
        tokio::select! {
            msg = socket.recv() => {
                let msg = match msg {
                    Some(Ok(Message::Text(text))) => text,
                    _ => break,
                };

                let parsed: EventClientMessage = match serde_json::from_str(&msg) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                match parsed {
                    EventClientMessage::Update { update } => {
                        state.event_vault.update_event(update.clone());

                        let _ = socket.send(Message::Text(
                            "Values updated successfully".into()
                        )).await;

                        let _ = state.event_tx.send(EventBroadcastUpdate {
                            sender_id: client_id,
                            updated_events: update,
                        });
                    }

                    EventClientMessage::Get { get } => {
                        subscribed_tags = get.clone();

                        // Збираємо поточні значення для підписаних тегів
                        let result: HashMap<String, serde_json::Value> = get
                            .iter()
                            .filter_map(|tag| {
                                state.event_vault
                                    .get_event(tag)
                                    .map(|v| (tag.clone(), v))
                            })
                            .collect();

                        send_json(&mut socket, &serde_json::json!({ "get_response": result })).await;
                    }
                }
            }

            Ok(update) = rx.recv() => {
                if update.sender_id == client_id {  // ← ось фільтр
                    continue;
                }
                if subscribed_tags.is_empty() {
                    continue;
                }

                let filtered: HashMap<&String, &serde_json::Value> = update
                    .updated_events
                    .iter()
                    .filter(|(tag, _)| subscribed_tags.contains(tag))
                    .collect();

                if !filtered.is_empty() {
                    send_json(&mut socket, &filtered).await;
                }
            }
        }
    }

    event(String::from("Відключення клієнта від events сокету"));
}

async fn send_json<T: Serialize>(socket: &mut WebSocket, data: &T) {
    if let Ok(json) = serde_json::to_string(data) {
        let _ = socket.send(Message::Text(json.into())).await;
    }
}