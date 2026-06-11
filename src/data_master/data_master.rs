use crate::messages::main_msg::MainMsg;
use tokio::sync::mpsc;
use crate::logger::*;
use crate::messages::commands::command::{Command, CommandType};

pub async fn run_data_master(mut from_api: mpsc::Receiver<MainMsg>, to_api: mpsc::Sender<MainMsg>, // TODO to_api change to broadcast!
                    mut from_reader: mpsc::Receiver<MainMsg>, to_reader: mpsc::Sender<CommandType>,
                    to_db: mpsc::Sender<MainMsg>,)
{
    printers::event(String::from("Старт контролера"));
    loop {
        tokio::select! {
            from_api_msg = from_api.recv() => {
                match from_api_msg {
                    Some(msg) => {
                        match msg {
                            MainMsg::Request(_) => {
                                send_to_db(msg, &to_db).await;
                            }
                            MainMsg::Command(command) => {
                                send_to_reader(command.cmd.clone(), &to_reader).await;
                                send_to_db(MainMsg::Command(command), &to_db).await;
                            }
                            MainMsg::Event(_) => { // API евентів не пакує
                                 printers::err(String::from("Пум-пум-пум... API не має відправляти MainMsg::Event!!!!"));
                            },
                        }
                    }
                    None => {
                        printers::err(String::from("Канал від API до контролера упав. Можна курить прямо тут"));
                        panic!()
                    }
                }
            }

            from_reader_msg = from_reader.recv() => {
                match from_reader_msg {
                    Some(msg) => {
                        match msg {
                            MainMsg::Request(_) => {
                                send_to_db(msg, &to_db).await;
                            },
                            MainMsg::Event(event) => {
                                send_to_db(MainMsg::Event(event.clone()), &to_db).await;
                                // send_api(MainMsg::Event(event), &to_api).await;  // TODO!!!!
                            },
                            MainMsg::Command(_) => { // в цій гілці не використовується
                                printers::err(String::from("Ти шо написав??! Читач не має відправляти MainMsg::Command!!!!"));
                            },
                        }
                    }
                    None => {
                        printers::err(String::from("Канал від опитувача до контролера упав. Це фіаско..."));
                        panic!()
                    }
                }
            }
        }
    };
}

async fn send_api(msg: MainMsg, to_api: &mpsc::Sender<MainMsg>) {
    let send_res = to_api.send(msg).await;
    match send_res {
        Ok(_) => {},
        Err(e) => {
            let msg = format!("Помилка перенаправлення від Контролера до API\n {e}");
            printers::warn(msg);
        }
    }
}

async fn send_to_db(msg: MainMsg, to_db: &mpsc::Sender<MainMsg>) {
    let send_res = to_db.send(msg).await;
    match send_res {
        Ok(_) => {
        },
        Err(e) => {
            let msg = format!("Помилка перенаправлення від Контролера до BD\n {e}");
            printers::warn(msg);
        }
    }
}

async fn send_to_reader(msg: CommandType, to_reader: &mpsc::Sender<CommandType>) {
    let send_res = to_reader.send(msg).await;
    match send_res {
        Ok(_) => {},
        Err(e) => {
            let msg = format!("Помилка перенаправлення від Контролера до опитувача\n {e}");
            printers::warn(msg);
        }
    }
}