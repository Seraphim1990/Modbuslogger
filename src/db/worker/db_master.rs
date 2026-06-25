use std::ops::Deref;
use sqlx::{MySql, Pool};
use tokio::sync::mpsc;
use crate::db::states::init_db;
use crate::messages::main_msg::MainMsg;
use crate::messages::requests::request_struct::Request;
use crate::db::worker::{
    node_worker::node_get,
    device_worker::devise_get,
    value_worker::value_get,
};
use crate::db::worker::device_worker::command_device;
use crate::db::worker::node_worker::command_node;
use crate::db::worker::value_worker::command_value;
use crate::logger::printers;
use crate::messages::commands::command::CommandType;
use crate::messages::events::{
    event::Event,
    device_event::DeviceEvent,
    node_event::NodeEvent,
};

use crate::db::hasher::hash_master::measure_master;
use crate::messages::config_event::ConfigEvent;

pub async fn run_db_master(rx: mpsc::Receiver<MainMsg>, tx_to_reader: mpsc::Sender<ConfigEvent>) {
    printers::event(String::from("Старт воркера бази даних"));

    let mut rx = rx;
    let pool = init_db(5).await;  // TODO зробити окремого воркера для роботи з Measure

    let (measure_tx, measure_rx) = mpsc::channel::<MainMsg>(100);

    tokio::spawn(measure_master(pool.clone(), measure_rx));

    while let Some(msg) = rx.recv().await {
        match msg {
            MainMsg::Command(msg) => {  // TODO
                match msg.cmd {
                    CommandType::NodeCommand(_) => {
                        command_node(&pool, msg, tx_to_reader.clone());
                    },
                    CommandType::DeviceCommand(_) => {
                        command_device(&pool, msg, tx_to_reader.clone());
                    },
                    CommandType::ValueCommand(_) => {
                        command_value(&pool, msg, tx_to_reader.clone());
                    },
                }
            },  //TODO
            MainMsg::Request(msg) => {
                match msg {
                    Request::GetNode(request) =>  node_get(&pool, request),
                    Request::GetDevice(request) => devise_get(&pool, request),
                    Request::GetValue(request) => value_get(&pool, request),
                    Request::GetDecodingType => {}, //TODO
                    Request::GetLogicGroup => {}, //TODO
                    Request::GetMeasure(measure) => {
                        let req = MainMsg::Request(Request::GetMeasure(measure));
                        if let Err(e) = measure_tx.send(req).await {
                            printers::err(format!("Помилка відправки події для зберігання: {}", e));
                        }
                    },
                }
            },
            MainMsg::Event(event) => {
                match event {
                    Event::DeviceEvent(dev_ev) => {
                        let ev = MainMsg::Event(Event::DeviceEvent(dev_ev));
                        if let Err(e) = measure_tx.send(ev).await {
                            printers::err(format!("Помилка відправки події для зберігання: {}", e));
                        }
                    },
                    Event::NodeEvent(_) => {}
                }
            }
        }
    }
    printers::warn(String::from("Воркер бази даних упав!!!!"));
}
