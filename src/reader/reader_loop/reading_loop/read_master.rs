// read_master.rs
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use crate::messages::{
    main_msg::MainMsg,
};
use tokio::sync::broadcast;
use crate::db::schemas::{
    node::NodeRead,
};
use crate::logger::printers;
use std::ops::Deref;

use crate::messages::commands::{
    command::Command,
    node::NodeCommand,
    device::DeviceCommand,
    value::ValueCommand,
};
use crate::reader::reader_loop::reading_loop::read_master_struct::ReadMaster;
use crate::messages::commands::command::CommandType;


pub async fn node_loop(to_controller: mpsc::Sender<MainMsg>, from_controller: broadcast::Receiver<CommandType>, conf: NodeRead) {
    let mut conf = conf;
    let to_controller = to_controller;
    let mut from_controller = from_controller;
    let mut read_master = ReadMaster::new(&conf, to_controller);

    let mut sleep_time = 0;

    loop {
            tokio::select! {
                msg = from_controller.recv() => {
                    match msg {
                        Ok(msg) => {
                            match msg {
                                CommandType::NodeCommand(node_cmd) => {
                                    match node_cmd.deref() {
                                                NodeCommand::Update(update_cmd) => {
                                                    if update_cmd.id == read_master.id() {
                                                        read_master.change_connecting_config(update_cmd.ip.clone(), update_cmd.port).await;
                                                    }
                                                },
                                                NodeCommand::Delete(delete_cmd) => {
                                                    if delete_cmd.id == read_master.id() {
                                                        printers::warn(format!("Закінчення роботи ноди ip: {}", read_master.ip()));
                                                        return;
                                                    }
                                                }
                                                NodeCommand::Create(_) => {
                                                    printers::err(String::from("Отримано сигнал NodeCreate в середині робочого таску!"));
                                                }
                                            }
                                },
                                CommandType::DeviceCommand(dev_cmd) => {
                                    match dev_cmd.deref() {
                                                DeviceCommand::Create(device_create) => {
                                                    if device_create.parent_node_id == read_master.id() {
                                                        read_master.add_device().await;
                                                    }
                                                },
                                                DeviceCommand::Delete(device_delete) => {
                                                    if device_delete.id == read_master.id() {
                                                        read_master.remove_device(device_delete.id);
                                                    }
                                                },
                                                DeviceCommand::Update(device_update) => {
                                                    read_master.update_devices(device_update.id).await;
                                                },
                                            }
                                },
                                CommandType::ValueCommand(value_cmd) => {
                                    match value_cmd.deref() {
                                                ValueCommand::Create(value_cmd) => {
                                                    read_master.value_create(value_cmd.parent_device_id).await;
                                                },
                                                ValueCommand::Delete(value_delete) => {
                                                    read_master.value_delete(value_delete.id).await;
                                                },
                                                ValueCommand::Update(value_update) => {
                                                    read_master.value_update(value_update).await;
                                                },
                                            }
                                },
                            }
                        }
                        Err(e) => {
                            let msg = format!("Помилка каналу від контроллера до циклу оритуванні: \n{}", e);
                            printers::err(msg);
                        }
                    }
                },
                _ = sleep(Duration::from_millis(sleep_time)) => {
                    read_master.tick().await;
                }
            }
        sleep_time = read_master.when_next().max(1);
    }
}