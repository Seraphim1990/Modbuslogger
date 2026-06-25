use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::messages::main_msg::MainMsg;
use crate::api::web_sockets::live_socket_unit::CoordUnitWebSocketData;
use crate::logger::printers;
use crate::messages::events::device_event::DeviceEventType;
use crate::messages::events::event::Event;
use crate::messages::events::node_event::NodeEventType;

pub async fn web_sock_coord(mut from_reader: mpsc::Receiver<MainMsg>, mut from_api: mpsc::Receiver<CoordUnitWebSocketData>) {

    let mut nodes: HashMap<i32, NodeEventType> = HashMap::new();
    let mut devises: HashMap<i32, DeviceEventType> = HashMap::new();
    let mut values: HashMap<Arc<String>, f64> = HashMap::new();

    let mut subscribers: Vec<CoordUnitWebSocketData> = Vec::new();

    loop {
        tokio::select! {
            from_reader_msg = from_reader.recv() => {

                match from_reader_msg {
                    Some(msg) => {
                        if let MainMsg::Event(event) = msg {
                            match event {
                                Event::NodeEvent(event) => {
                                    if check_change(&mut nodes, event.id, event.event.clone()) {
                                        for subscriber in subscribers.iter_mut() {
                                            subscriber.node_events(event.id, &event.event)
                                        }
                                        flush_subscribers(&mut subscribers);
                                    }
                                },
                                Event::DeviceEvent(event) => {
                                    if check_change(&mut devises, event.id, event.event.clone()) {
                                        for subscriber in subscribers.iter_mut() {
                                            subscriber.device_events(event.id, &event.event)
                                        }
                                    }
                                    for value in &event.measures {
                                        if check_change(&mut values, value.tag.clone(), value.measure_value) {
                                            for subscriber in subscribers.iter_mut() {
                                                subscriber.value_events(value.tag.clone(), value.measure_value)
                                            }
                                        }
                                    }
                                    flush_subscribers(&mut subscribers);
                                },
                            }
                        } else {
                            printers::err("web_sock_coord не повинен приймати нічого крім MainMsg::Event".to_string());
                        }
                    },
                    None => {
                        printers::warn("Падіння каналу від головного контролера в маршрутизаторі вебсокетів".to_string());
                        panic!() // головний контролер упав, він має працювати завжди
                    }
                }
            }
            from_api_msg = from_api.recv() => {
                match from_api_msg {
                    Some(subscriber) => {
                        let mut subscriber = subscriber;
                        for (key, val) in nodes.iter() {
                            subscriber.node_events(*key, val);
                        }
                        for (key, val) in devises.iter() {
                            subscriber.device_events(*key, val);
                        }
                        for (key, val) in values.iter() {
                            subscriber.value_events(key.clone(), *val);
                        }
                        if subscriber.flush() {
                            subscribers.push(subscriber);
                        }
                    },
                    None => {
                        printers::warn("Падіння каналу від API в маршрутизаторі вебсокетів".to_string());
                    }
                }
            }
        }
    }
}

fn check_change<K, V>(target: &mut HashMap<K, V>, key: K, value: V) -> bool
where
    K: Eq + Hash,
    V: PartialEq,
{
    match target.entry(key) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            if *entry.get() == value {
                false
            } else {
                entry.insert(value);
                true
            }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            entry.insert(value);
            true
        }
    }
}

fn flush_subscribers(subscribers: &mut Vec<CoordUnitWebSocketData>) {
    subscribers.retain_mut(|sub| sub.flush());  // retain_mut Дозволяє змінювати (мутувати) елементи безпосередньо під час їх фільтрації
}