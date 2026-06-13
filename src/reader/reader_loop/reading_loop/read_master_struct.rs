// read_master_struct.rs
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, timeout};
use tokio_modbus::client::{tcp, Context};
use tokio_modbus::{Error, ExceptionCode, ProtocolError, Slave};
use tokio_modbus::prelude::{Reader, SlaveContext};
use crate::messages::main_msg::MainMsg;
use crate::reader::reader_loop::modbus_device::modbus_device::ModbusDeviceUnit;

use crate::db::schemas::{
    node::NodeRead,
    device::DeviceRead,
    value_unit::ValueRead
};
use crate::db::schemas::device::DeviceCreate;
use crate::db::schemas::value_unit::ValueUpdate;
use crate::logger::printers;
use crate::messages::requests::device_request::{DeviceRequest, GetDeviceByNode, GetDeviceById};
use crate::messages::requests::request_struct::Request;
use crate::messages::requests::value_request::{GetByDeviceId, ValueRequest};
use crate::messages::events::{
event::Event,
node_event::{NodeEvent, NodeEventType},
device_event::{DeviceEvent, DeviceEventType}
};
use crate::reader::reader_loop::decoding_plugins::value_interface::RegType;

#[derive(Debug, Clone, Copy)]
enum ModbusReadError {
    Timeout,
    DeviceProtocolError,
    SocketFail,
    RemoveStep,
    HeaderMisMatch
}

type ModbusReadResult =
Result<Vec<u16>, ModbusReadError>;

pub struct ReadMaster{
    id: i32,
    ip: Arc<String>,
    port: i32,
    devices: Vec<ModbusDeviceUnit>,
    ctx: Option<Context>,
    current_devise_index: Option<usize>,
    to_controller: mpsc::Sender<MainMsg>,
    ctx_state: NodeEventType,
    last_connecting_time: u64,
}
impl ReadMaster{
    pub fn new(config: &NodeRead, sender: mpsc::Sender<MainMsg>) -> ReadMaster{
        ReadMaster{
            id: config.id,
            ip: Arc::new(config.ip.clone()),
            port: config.port.unwrap_or(502),
            devices: Vec::new(),
            ctx: None,
            current_devise_index: None,
            to_controller: sender,
            ctx_state: NodeEventType::UnConnected,
            last_connecting_time: 0
        }
    }
    pub fn id(&self) -> i32{
        self.id
    }
    pub fn ip(&self) -> Arc<String>{
        self.ip.clone()
    }
    pub async fn change_connecting_config(&mut self, ip: Option<String>, port: Option<i32>){
        let mut changed = false;
        if let Some(ip) = ip {
            if self.ip.as_str() != ip {
                self.ip = Arc::new(ip);
                changed = true;
            }
        }
        if let Some(port) = port {
            if self.port != port {
                self.port = port;
                changed = true;
            }
        }
        if changed {
            let _ =self.create_context().await;
        }
    }
    pub async fn add_device(&mut self){
        let mut counter = 0;
        loop {
            sleep(Duration::from_millis(50)).await; // чекаємо доки збережеться в БД
            let devices = self.get_devises().await;
            if let Some(device) = devices
                .into_iter()
                .find(|d| !self.devices.iter().any(|u| u.id() == d.id)) {
                let new_device = self.get_device_from_db(device).await;
                if let Some(new_device) = new_device {
                    self.devices.push(new_device);
                    break;
                }
            }
            counter += 1;
            if counter > 5 {
                printers::warn(format!("Не вдалось оновити конфігурацію {}", self.ip));
                break;
            }
        }
    }

    pub fn remove_device(&mut self, id: i32){
        if let Some(index) = self.devices.iter().position(|d| d.id() == id){
            self.devices.remove(index);
        }
    }

    pub async fn update_devices(&mut self, id: i32){
        let mut counter = 0;
        loop {
            sleep(Duration::from_millis(50)).await; // чекаємо доки збережеться в БД
            if let Some(index) = self.devices.iter().position(|d| d.id() == id) {
                if let Some(values) = self.get_values_from_db(self.devices[index].id()).await {
                    self.devices[index].set_values(values);
                    break;
                }
            }
            counter += 1;
            if counter > 5 {
                printers::warn(format!("Не вдалось оновити конфігурацію {}", self.ip));
                break;
            }
        }
    }

    pub async fn value_create(&mut self, parent_dev_id: i32) {
        if self.devices.iter().any(|d| d.id() == parent_dev_id) {
            self.update_devices(parent_dev_id).await;
        }
    }
    pub async fn value_delete(&mut self, value_id: i32){
        if let Some(device) = self.devices.iter().find(|d| d.contains_value_id(value_id)) {
            self.update_devices(device.id()).await;
        }
    }

    pub async fn value_update(&mut self, value_conf: &ValueUpdate){
        let mut old = None;
        if let Some(device) = self.devices.iter().find(|d| d.contains_value_id(value_conf.id)) {
            old = Some(device.id());
            self.update_devices(old.unwrap()).await;
        }
        if let Some(new) = value_conf.parent_device_id {
            if let Some(old) = old {
                if old == new {
                    return;
                }
            }
            if self.devices.iter().any(|d| d.id() == new) {
                self.update_devices(new).await;
            }
        }
    }

    pub fn when_next(&mut self) -> u64 {
        let ts = Self::get_time();
        if self.devices.len() == 0 {
            return 1000; // пристроїв не знайдено, спробуємо через секунду, може шото з команд прийде...
        };
        if self.ctx.is_none() {
            if let Some(next) = self.last_connecting_time.checked_sub(ts) {
                return next
            } else {
                return 0
            }
        };

        if let Some(next) = self.when_next_device().checked_sub(ts) {
            next
        } else {
            0
        }
    }

    pub async fn tick(&mut self) {
        if self.devices.len() == 0 {
            if!(self.create_devices().await) {
                return; // пристроїв не знайдено, спробуємо через секунду, може шото з команд прийде...
            };
        };
        // пристрої є, шо там з контекстом?
        if self.ctx.is_none() {
            if !self.create_context().await{
                return; // шото не то с контекстом, попробуємо наступного разу...
            }
        };

        self.read().await;
    }

    async fn read(&mut self) {
        let mut device = &mut self.devices[self.current_devise_index.unwrap()];

        if !device.is_active() {
            return;
        }

        let ctx = match self.ctx.as_mut() {
            Some(ctx) => ctx,
            None => return,
        };
        let slave = Slave(device.address());
        ctx.set_slave(slave);

        let dev_id = device.id();
        let total_steps = device.total_steps();

        let mut total_result = Vec::new();
        let mut timeout_duration = device.timeout();
        let retry_count = device.retry_count() as u64;

        let mut failed_steps = 0;

        let mut is_increase_timeout = false;

        for step in device.get_pool() {
            if step.is_broken() {
                failed_steps += 1;
                continue
            };
            let mut curr_try = 0;
            loop {
                let read_result = match step.get_type(){
                    RegType::Input => {
                        read_process(|| ctx.read_input_registers(step.get_start(), step.get_length()),
                                                                                            timeout_duration,
                                                                                            dev_id,
                                                                                            self.ip.clone(),
                        ).await
                    },
                    RegType::Holding => {
                        read_process(|| ctx.read_holding_registers(step.get_start(), step.get_length()),
                                     timeout_duration,
                                     dev_id,
                                     self.ip.clone(),
                        ).await
                    }
                    RegType::Coils => {
                        read_process(|| ctx.read_coils(step.get_start(), step.get_length()),
                                     timeout_duration,
                                     dev_id,
                                     self.ip.clone(),
                        ).await
                    },
                    RegType::Discrete => {
                        read_process(|| ctx.read_discrete_inputs(step.get_start(), step.get_length()),
                                     timeout_duration,
                                     dev_id,
                                     self.ip.clone(),
                        ).await
                    },
                };

                match read_result {
                    Ok(read_result) => {
                        for val in step.get_value(&read_result) {
                            total_result.push(val);
                        }
                        break;
                    },
                    Err(e) => {
                        match e {
                            ModbusReadError::Timeout | ModbusReadError::DeviceProtocolError => {
                                curr_try += 1;
                            },
                            ModbusReadError::SocketFail => {
                                self.disconnecting().await;
                                return;
                            },
                            ModbusReadError::RemoveStep => {
                                step.mark_broken();
                                printers::err(format!("Заблоковані регістри пристрою id: {}, адреси з {} по {}  для ноди :{}",
                                                      dev_id,
                                                      step.get_start(),
                                                      (step.get_start() + step.get_length()) - 1,
                                                      self.ip.clone()));
                                for val in step.fail() {
                                    total_result.push(val);
                                }
                                failed_steps += 1;
                                break;
                            },
                            ModbusReadError::HeaderMisMatch => {
                                if curr_try > 0 { // фільтрація на опитування попередніми пристроями!
                                    is_increase_timeout = true;
                                    printers::warn(format!("Збільшено таймаут для пристрою id: {}, адреса: {}, нода: {}", device.id(), device.address(), self.ip.clone()));
                                    sleep(Duration::from_millis(50)).await;
                                    break;
                                }
                            }
                        }
                    }
                }
                if curr_try >= retry_count {
                    for val in step.fail() {
                        total_result.push(val);
                    }
                    failed_steps += 1;
                    break;
                }
            };
        }
        if is_increase_timeout {
            device.increase_timeout();
        }

        let read_report = match failed_steps {
            0 => DeviceEventType::Full,
            _ if failed_steps == total_steps => DeviceEventType::Failed,
            _ => DeviceEventType::SomePart,
        };

        let res = DeviceEvent{
            event: read_report,
            id: dev_id,
            measures: total_result,
        };

        let res = MainMsg::Event(
            Event::DeviceEvent(
                Arc::new(res),
            )
        );
        device.finished();
        let send_result = self.to_controller.send(res).await;
        match send_result {
            Ok(_) => {
            },
            Err(e) => {
                printers::err(format!("Помилка відправки результатів опитування: {:?}", e));
            }
        }
    }

    fn when_next_device(&mut self) -> u64{
        let mut when_next = self.devices[0].when_next();  // при викликові цього методу self.devices.len() != 0 і контекст має бути створений!
        self.current_devise_index = Some(0);
        for i in 0..self.devices.len() {
            if self.devices[i].when_next() < when_next && self.devices[i].is_active() {
                when_next = self.devices[i].when_next();
                self.current_devise_index = Some(i);
            };
        };

        if self.devices[self.current_devise_index.unwrap_or(0)].is_active() {  // якшо немає активних то просто спимо 1 сек
            when_next
        } else {
            1000
        }
    }

    async fn create_context(&mut self) -> bool {
        self.ctx = None;  // про всяк випадок чистимо старий контекст
        self.start_connecting().await;
        let socket_addr = match format!("{}:{}", self.ip, self.port).parse() {
            Ok(addr) => addr,
            Err(e) => {
                printers::err(format!("Не вірна адреса сокету: {}", e));
                return false;
            }
        };
        let res = tcp::connect(socket_addr).await;

        match res {
            Ok(ctx) => {
                let msg = format!("Створено контекст для ip: {}", &self.ip);
                printers::event(msg);
                self.ctx = Some(ctx);
                self.connecting_finished().await;
                true
            },
            Err(e) => {
                let msg = format!("Помилка створення контексту для ip: {}, \nerr: {} \nПовторна спроба через 60 сек.", &self.ip, e);
                printers::err(msg);
                self.disconnecting().await;
                false
            }
        }
    }

    async fn create_devices(&mut self) -> bool { // true = devices was created
        let devices = self.get_devises().await;
        if devices.len() == 0 {
            return false;
        }
        let mut devices_units = Vec::with_capacity(devices.len());
        for device_config in devices {
            let mut new_dev_unit = self.get_device_from_db(device_config).await;
            if let Some(new_dev_unit) = new_dev_unit {
                devices_units.push(new_dev_unit);
            }
        }
        self.devices = devices_units;
        let _ =self.when_next_device();
        true
    }

    async fn get_device_from_db(&mut self, device_config: DeviceRead) -> Option<ModbusDeviceUnit> {
        let mut new_dev_unit = ModbusDeviceUnit::new(device_config);
        let values = self.get_values_from_db(new_dev_unit.id()).await;
        if let Some(values) = values {
            new_dev_unit.set_values(values);
            Some(new_dev_unit)
        } else {
            None
        }
    }

    async fn get_values_from_db(&mut self, id: i32) -> Option<Vec<ValueRead>> {
        let (tx, rx) = oneshot::channel::<Result<Vec<ValueRead>, ()>>();
        let send_res = self.to_controller.send(Self::create_value_request(id, tx)).await;
        match send_res {
            Ok(_) => {},
            Err(e) => {
                printers::err(format!("Помилка відправки запиту до БД на отримання значень\nid:{}\nErr: {}", id, e));
                return None;
            }
        }
        let res = rx.await;
        let values = match res {
            Ok(msg) => {
                match msg {
                    Ok(values_res) => {
                        values_res
                    },
                    Err(_) => {
                        printers::err(format!("Помилка  отримання значень\nid:{}\nПомилка бази даних", id));
                        return None;
                    }
                }
            },
            Err(e) => {
                printers::err(format!("Помилка каналу на отримання значень\nid:{}\nErr: {}", id, e));
                return None;
            }
        };
        Some(values)
    }

    async fn get_devises(&mut self) -> Vec<DeviceRead> {
        let mut counter = 0;
        let mut devices = Vec::new();

        loop {
            let (tx, rx) = oneshot::channel();

            let dev_request = GetDeviceByNode {
                node_id: self.id,
                request_channel: tx,
            };
            let dev_request = DeviceRequest::GetByNode(dev_request);
            let dev_request = Request::GetDevice(dev_request);
            let dev_request = MainMsg::Request(dev_request);

            let send_res = self.to_controller.send(dev_request).await;
            match send_res {
                Ok(_) => {},
                Err(e) => {
                    let msg = format!("Помилка відправки повідомлення до контролера при створенні пристроїв для {}\nErr: {}", &self.ip, e.to_string());
                    printers::err(msg);
                }
            }

            let res = rx.await;

            match res {
                Ok(msg) => {
                    match msg {
                        Ok(devices_res) => {
                            devices = devices_res;
                            break;
                        },
                        Err(_) => {
                            let msg = format!("Помилка отримання даних від бази даних для {}", &self.ip);
                            printers::err(msg);
                        }
                    }
                },
                Err(e) => {
                    let msg = format!("Помилка каналу отримування відповіді для {} від бази: {}", &self.ip, e);
                    printers::err(msg);
                }
            }
            sleep(std::time::Duration::from_millis(100)).await;
            counter += 1;
            if counter >= 5 {
                let msg = format!("Помилка отримання пристроїв для {}, кількість спроб вичерпано", &self.ip);
                printers::warn(msg);
            }
        }
        devices
    }

    async fn get_one_device_form_db(&mut self, id: i32) -> Option<ModbusDeviceUnit> {
        let (tx, rx) = oneshot::channel();

        let dev_request = GetDeviceById {
            id: self.id,
            request_channel: tx,
        };
        let dev_request = DeviceRequest::GetDeviceById(dev_request);
        let dev_request = Request::GetDevice(dev_request);
        let dev_request = MainMsg::Request(dev_request);
        let send_res = self.to_controller.send(dev_request).await;
        match send_res {
            Ok(_) => {},
            Err(e) => {
                let msg = format!("Помилка відправки повідомлення до контролера при створенні пристроїв для {}\nErr: {}", &self.ip, e.to_string());
                printers::err(msg);
            }
        }

        let res = rx.await;

        let device = match res {
            Ok(msg) => {
                match msg {
                    Ok(dev_res) => {
                        match dev_res {
                            Some(dev_) => dev_,
                            None => return None
                        }
                    }
                    Err(e) => {
                        let msg = format!("Помилка отримання даних від бази даних для {}", &self.ip);
                        printers::err(msg);
                        return None
                    }
                }
            },
            Err(e) => {
                let msg = format!("Помилка каналу отримування відповіді для {} від бази: {}", &self.ip, e);
                printers::err(msg);
                return None
            }
        };

        if let Some(new_device) = self.get_device_from_db(device).await {
            Some(new_device)
        } else {
            None
        }
    }

    fn create_value_request(dev_id: i32, chan: oneshot::Sender<Result<Vec<ValueRead>, ()>>) -> MainMsg {
        let val_request = GetByDeviceId {
            device_id: dev_id,
            request_channel: chan,
        };
        let val_request = ValueRequest::GetByDeviceId(val_request);
        let val_request = Request::GetValue(val_request);
        MainMsg::Request(val_request)
    }
    fn get_time() -> u64 {
        let sys_time = SystemTime::now()
            .duration_since(UNIX_EPOCH);
        match sys_time {
            Ok(duration) => {
                duration.as_millis() as u64
            }
            Err(_) => {
                printers::warn(String::from("Помилка отримання часової відмітки"));
                0
            }
        }
    }

    async fn start_connecting(&mut self) {
        self.send_connecting_msg(NodeEventType::Connecting).await;
    }
    async fn connecting_finished(&mut self) {
        self.send_connecting_msg(NodeEventType::Connected).await;
    }

    async fn disconnecting(&mut self) {
        self.last_connecting_time = Self::get_time() + 60000;
        self.send_connecting_msg(NodeEventType::UnConnected).await;
    }
    async fn send_connecting_msg(&mut self, msg: NodeEventType) {
        self.ctx_state = msg;

        let send_msg = MainMsg::Event(
            Event::NodeEvent(
                Arc::new(
                    NodeEvent{
                        event: msg,
                        id: self.id,
                        ip: self.ip.clone(),
                    }
                )
            )
        );
        let send_res = self.to_controller.send(send_msg).await;
        match send_res {
            Ok(_) => {},
            Err(e) => {
                let msg = format!("Помилка відправки повідомлення до контролера про початок створення {}\nErr: {}", &self.ip, e.to_string());
                printers::err(msg);
            }
        }
    }
}

async fn read_process<F, Fut, Word>(
    f: F,
    timeout_duration: u64,
    device_id: i32,
    node_ip: Arc<String>,
) -> ModbusReadResult
where
    F: FnOnce() -> Fut,
// Трейт-баунд під будь-яку таску читання з tokio_modbus
    Fut: Future<Output = Result<Result<Vec<Word>, ExceptionCode>, Error>>,
// Гарантуємо, що Word (u16 або bool) можна скастити в u16
    Word: Into<u16> + Copy,
{
    let timeout_duration = Duration::from_millis(timeout_duration);
    let res = timeout(timeout_duration, f()).await;
    match res {
        Ok(Ok(data)) => {
            match data {
                Ok(raw_values) => {
                    let cleaned_data = raw_values.iter().map(|&val| val.into()).collect::<Vec<u16>>();
                    Ok(cleaned_data)
                },
                Err(e) => {
                    match e {
                        ExceptionCode::IllegalFunction => {
                            printers::err(format!("Пристрій не підтримує функцію запиту\n    id: {} нода {}", device_id, node_ip));
                            Err(ModbusReadError::RemoveStep)
                        },
                        ExceptionCode::IllegalDataAddress => {
                            printers::err(format!("Запитані регістри не існують\n    id: {} нода {}", device_id, node_ip));
                            Err(ModbusReadError::RemoveStep)
                        },
                        ExceptionCode::IllegalDataValue | ExceptionCode::MemoryParityError => {
                            printers::err(format!("Біда з даними/паритетом (CRC?)\n    id: {} нода {}", device_id, node_ip));
                            Err(ModbusReadError::Timeout) // Повертаємо як таймаут, нехай пробує ще
                        },
                        ExceptionCode::ServerDeviceBusy | ExceptionCode::Acknowledge => {
                            // Пристрій зайнятий. Спимо 100 мс прямо тут і повертаємо Timeout для ретраю
                            Err(ModbusReadError::Timeout)
                        },
                        ExceptionCode::ServerDeviceFailure | ExceptionCode::GatewayPathUnavailable | ExceptionCode::GatewayTargetDevice => {
                            // Щось із залізом або шлюзом. Шлемо на перепідключення сокета
                            Err(ModbusReadError::SocketFail)
                        },
                        _ => Err(ModbusReadError::Timeout),
                    }
                }
            }
        }
        Ok(Err(e)) => {
            match e {
                Error::Transport(io_err) => {
                    match io_err.kind() {
                        ErrorKind::ConnectionRefused  |
                        ErrorKind::ConnectionReset    |
                        ErrorKind::HostUnreachable    |
                        ErrorKind::NetworkUnreachable |
                        ErrorKind::ConnectionAborted  |
                        ErrorKind::NotConnected       |
                        ErrorKind::BrokenPipe         |
                        ErrorKind::TimedOut => {
                            // Відправляємо сигнал на create_context
                            Err(ModbusReadError::SocketFail)
                        },

                        ErrorKind::NetworkDown => {
                            printers::err(String::from("Упала мережа системи,"));
                            Err(ModbusReadError::SocketFail)
                        },

                        other => {
                            printers::warn(format!("Дивна помилка IO при читанні ModbusTCP:\n {:?}", other));
                            Err(ModbusReadError::Timeout)
                        }
                    }
                },
                Error::Protocol(protocol_error) => {
                    match protocol_error {
                        ProtocolError::FunctionCodeMismatch { .. } => {
                            printers::warn(format!("Код відповіді не відповідає запиту\n id: {}, для ноди {}", device_id, node_ip));
                            Err(ModbusReadError::DeviceProtocolError)
                        },
                        ProtocolError::HeaderMismatch { message, .. } => {
                            printers::warn(format!("Заголовок відповіді не відповідає запиту: {}\n id: {}, для ноди {}", message, device_id, node_ip));
                            Err(ModbusReadError::HeaderMisMatch)
                        },
                    }
                }
            }
        }
        Err(_) => {
            Err(ModbusReadError::Timeout)
        }
    }
}