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

pub async fn run_db_master(rx: mpsc::Receiver<MainMsg>) {
    printers::event(String::from("Старт воркера бази даних"));

    let mut rx = rx;
    let pool = init_db(5).await;  // TODO зробити окремого воркера для роботи з Measure

    while let Some(msg) = rx.recv().await {
        match msg {
            MainMsg::Command(msg) => {  // TODO
                match msg.cmd {
                    CommandType::NodeCommand(_) => {
                        command_node(&pool, msg);
                    },
                    CommandType::DeviceCommand(_) => {
                        command_device(&pool, msg);
                    },
                    CommandType::ValueCommand(_) => {
                        command_value(&pool, msg);
                    },
                }
            },  //TODO
            MainMsg::Request(msg) => {
                match msg {
                    Request::GetNode(request) =>  node_get(&pool,request),
                    Request::GetDevice(request) => devise_get(&pool,request),
                    Request::GetValue(request) => value_get(&pool,request),
                    Request::GetDecodingType => {}, //TODO
                    Request::GetLogicGroup => {}, //TODO
                    Request::GetMeasure(measure) => {}, //TODO
                }
            },
            MainMsg::Event(event) => {
                match event {
                    Event::DeviceEvent(dev_ev) => {
                        printers::event(String::from("Отримано дані вимірювання"));
                        for i in &dev_ev.measures {
                            println!("{:?}", i);
                        }
                    },
                    Event::NodeEvent(node_ev) => {} // TODO!
                }
            }
        }
    }
    printers::warn(String::from("Воркер бази даних упав!!!!"));
}
