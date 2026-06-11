use sqlx::{MySql, Pool};
use crate::db::schemas::device::*;
use crate::db::schemas::node::NodeRead;
use crate::messages::requests::device_request::*;
use crate::logger::printers;
use crate::messages::commands::command::{Command, CommandType};
use crate::messages::commands::device::DeviceCommand;

pub fn command_device(pool: &Pool<MySql>, command: Command) {
    let pool = pool.clone();
    if let CommandType::DeviceCommand(device) = command.cmd  {
        let tx = command.request_channel;
        match device.as_ref() {
            DeviceCommand::Create(_) => {
                let device = device.clone();
                tokio::spawn(async move {
                    if let DeviceCommand::Create(device) = device.as_ref() {
                        let res = create_device(&pool, device).await;
                        if tx.send(res).is_err() {
                            let msg = "повернення калбеку DeviceCommand::Create".to_string();
                            printers::err(msg)
                        }
                    }
                });
            },
            DeviceCommand::Delete(_) => {
                let device = device.clone();
                tokio::spawn(async move {
                    if let DeviceCommand::Delete(device) = device.as_ref() {
                        let res = delete_device(&pool, device).await;
                        if tx.send(res).is_err() {
                            let msg = format!("повернення калбеку DeviceCommand::Delete id: {}", device.id);
                            printers::err(msg)
                        }
                    }
                });
            },
            DeviceCommand::Update(_) => {
                let device = device.clone();
                tokio::spawn(async move {
                    if let DeviceCommand::Update(device) = device.as_ref() {
                        let res = update_device(&pool, device).await;
                        if tx.send(res).is_err() {
                            let msg = format!("повернення калбеку DeviceCommand::Update id: {}", device.id);
                            printers::err(msg)
                        }
                    }
                });
            },
        }
    }
}

pub fn devise_get(pool: &Pool<MySql>, request: DeviceRequest) {
    let pool = pool.clone();

    match request {
        DeviceRequest::GetDeviceById(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_device_by_id(&pool, request.id).await;
                if tx.send(res).is_err() {
                    let msg = format!("Помилка відправки DeviceRequest::GetDeviceById {}", request.id);
                    printers::warn(msg);
                };
            });
        }
        DeviceRequest::GetByNode(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_device_by_node_id(&pool, request.node_id).await;
                if  tx.send(res).is_err() {
                    let msg = format!("Помилка відправки DeviceRequest::GetByNode {}", request.node_id);
                    printers::warn(msg);
                };
            });
        }
        DeviceRequest::GetAllDevices(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_all_devices(&pool).await;
                if tx.send(res).is_err() {
                    printers::warn(String::from("Помилка відправки DeviceRequest::GetAllDevices"));
                };
            });
        }
        DeviceRequest::GetDeleted(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_deleted_devices(&pool).await;
                let send_res = tx.send(res);
                match send_res {
                    Ok(_) => {},
                    Err(e) => {
                        printers::warn(String::from("Помилка відправки DeviceRequest::GetDeleted"));
                    }
                };
            });
        }
    }
}

async fn get_device_by_id(pool: &Pool<MySql>, id: i32) -> Result<Option<DeviceRead>, ()> {
    let device = sqlx::query_as::<_, DeviceRead>(
        "SELECT id,
       parent_node_id,
       device_name,
       address,
       time_for_recall,
       timeout,
       retry_count,
       is_active,
       read_by_group,
       description,
       deleted,
       deleted_at
        FROM devices WHERE id = ? AND deleted = 0",
    ).bind(id).fetch_optional(pool).await;
    match device {
        Ok(Some(device)) => {Ok(Some(device))},
        Ok(None) => {Ok(None)},
        Err(err) => {
            let msg = format!("Помилка отримання пристрою по id: \n{:?}", err);
            printers::err(msg);
            Err(())
        }
    }
}

async fn get_device_by_node_id(pool: &Pool<MySql>, parent_id: i32) -> Result<Vec<DeviceRead>, ()> {
    let devices = sqlx::query_as::<_, DeviceRead>(
        "SELECT id,
       parent_node_id,
       device_name,
       address,
       time_for_recall,
       timeout,
       retry_count,
       is_active,
       read_by_group,
       description,
       deleted,
       deleted_at
        FROM devices WHERE parent_node_id = ? AND deleted = 0"
    )
        .bind(parent_id)
        .fetch_all(pool)
        .await;

    match devices {
        Ok(devices) => Ok(devices),
        Err(err) => {
            let msg = format!("Помилка отримання пристрою по батьківському id: \n{:?}", err);
            printers::err(msg);
            Err(())
        }
    }
}

async fn get_all_devices(pool: &Pool<MySql>) -> Result<Vec<DeviceRead>, ()> {
    let devices = sqlx::query_as::<_, DeviceRead>(
        "SELECT id,
       parent_node_id,
       device_name,
       address,
       time_for_recall,
       timeout,
       retry_count,
       is_active,
       read_by_group,
       description,
       deleted,
       deleted_at
        FROM devices"
    )
        .fetch_all(pool)
        .await;
    match devices {
        Ok(devices) => Ok(devices),
        Err(err) => {
            let msg = format!("Помилка отримання усих пристроїв: \n{:?}", err);
            printers::err(msg);
            Err(())
        }
    }
}

async fn get_deleted_devices(pool: &Pool<MySql>) -> Result<Vec<DeviceRead>, ()> {
    let devices = sqlx::query_as::<_, DeviceRead>(
        "SELECT id,
       parent_node_id,
       device_name,
       address,
       time_for_recall,
       timeout,
       retry_count,
       is_active,
       read_by_group,
       description,
       deleted,
       deleted_at
        FROM devices WHERE deleted = 1"
    ).fetch_all(pool).await;
    match devices {
        Ok(devices) => Ok(devices),
        Err(err) => {
            let msg = format!("Помилка отримання видалених пристроїв: \n{:?}", err);
            printers::err(msg);
            Err(())
        }
    }
}

async fn delete_device(pool: &Pool<MySql>, device: &DeviceDelete) -> Result<(), String> {
    sqlx::query("DELETE FROM devices WHERE id = ?")
    .bind(device.id)
    .execute(pool)
    .await
        .map_err(|err| {
            let msg = format!("Помилка видалення пристрою id: {}\n{}", device.id, err.to_string());
            printers::err(msg.clone());
            msg
        })?;
    let msg = format!("Видалено пристрій по запиту: {:?}", device);
    printers::event(msg);
    Ok(())
}

async fn create_device(pool: &Pool<MySql>, device: &DeviceCreate) -> Result<(), String> {

    if device.address < 0 || device.address > 255 {
        return Err("Не вірна адреса пристрою".to_string());
    }
    if device.time_for_recall <= 0 {
        return Err("Час опитування не може бути від'ємним або нулем".to_string());
    }
    if device.retry_count <= 0 {
        return Err("Кількість повторів не може бути від'ємною або нулем".to_string());
    }
    if device.timeout <= 0 {
        return Err("Час таймауту (timeout) не може бути від'ємним або нулем".to_string());
    }
    if device.parent_node_id <= 0 {
        return Err("id Батьківської ноди не може бути від'ємним".to_string());
    }

    let node_read = sqlx::query_as::<_, NodeRead>("SELECT id, ip, port, description FROM nodes WHERE id = ?")
        .bind(device.parent_node_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            let msg = format!("Помилка отримання ноди від БД по id: {:?}", e);
            printers::err(msg.clone());
            msg
        })?;
    if node_read.is_none() {
        let msg = format!("Помилка створення пристрою, відсутня батьківська нода: {:?}", device);
        printers::err(msg.clone());
        return Err(msg)
    }


    let device_read = sqlx::query_as::<_, DeviceRead>(
        "SELECT id, parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description, deleted, deleted_at
             FROM devices
             WHERE address = COALESCE(?, address) AND parent_node_id = COALESCE(?, parent_node_id) AND deleted = 0"
    )
        .bind(device.address)
        .bind(device.parent_node_id)
        .fetch_optional(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка перевірки унікальності адреси: {}", e);
            printers::err(msg.clone());
            msg
        })?;

    if let Some(_) = device_read {
        return Err("Ця нода уже має пристрій з такою адресою".to_string());
    }

    let description = match &device.description {
        Some(description) => &description,
        None => &"".to_string(),
    };

    sqlx::query(
        "INSERT INTO devices (parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(device.parent_node_id)
        .bind(&device.device_name)
        .bind(device.address)
        .bind(device.time_for_recall)
        .bind(device.timeout)
        .bind(device.retry_count)
        .bind(device.is_active)
        .bind(device.read_by_group)
        .bind(description)
        .execute(pool)
        .await
        .map_err(|e| {
            let msg = format!("Помилка створення пристрою: {:?}", e);
            printers::err(msg.clone());
            msg
        })?;

    let msg = format!("Створено пристрій по запиту: {:?}", device);
    printers::event(msg);

    Ok(())
}

async fn update_device(pool: &Pool<MySql>, device: &DeviceUpdate) -> Result<(), String> {
    if let Some(addr) = device.address {
        if addr < 0 || addr > 255 {
            return Err("Не вірна адреса пристрою".to_string());
        }
    }
    if let Some(recall) = device.time_for_recall {
        if recall < 0 {
            return Err("Час опитування не може бути від'ємним".to_string());
        }
    }
    if let Some(timeout) = device.timeout {
        if timeout < 0 {
            return Err("Час таймауту не може бути від'ємним".to_string());
        }
    }
    if let Some(retry) = device.retry_count {
        if retry < 0 {
            return Err("Кількість повторів не може бути від'ємною".to_string());
        }
    }

    // Якщо прилітає parent_node_id, перевіряємо його на валідність та існування ноди
    if let Some(p_node_id) = device.parent_node_id {
        if p_node_id <= 0 {
            return Err("id Батьківської ноди не може бути від'ємним".to_string());
        }

        let node = sqlx::query_as::<_, NodeRead>(
            "SELECT id, ip, port, description FROM nodes WHERE id = ?"
        )
            .bind(p_node_id)
            .fetch_optional(pool)
            .await
            .map_err(|e|{
                let msg = format!("Помилка бази даних при перевірці ноди: \n{}", e);
                printers::err(msg.clone());
                msg
            })?;
        if node.is_none() {
            return Err("Ноди з таким id не існує".to_string());
        }
    }

    if device.address.is_some() || device.parent_node_id.is_some() { // TODO розширити логіку для перевірки унікальності
        let device_read = sqlx::query_as::<_, DeviceRead>(
            "SELECT id, parent_node_id, device_name, address, time_for_recall, timeout, retry_count, is_active, read_by_group, description, deleted, deleted_at
             FROM devices
             WHERE address = COALESCE(?, address) AND parent_node_id = COALESCE(?, parent_node_id) AND deleted = 0"
        )
            .bind(device.address)
            .bind(device.parent_node_id)
            .fetch_optional(pool)
            .await
            .map_err(|e|{
                let msg = format!("Помилка перевірки унікальності адреси: {}", e);
                printers::err(msg.clone());
                msg
            })?;

        if let Some(device_read) = device_read {
            if device_read.id != device.id as i32 {
                return Err("Ця нода уже має пристрій з такою адресою".to_string());
            }
        }
    }
    sqlx::query(
        "UPDATE devices
         SET
            device_name = COALESCE(?, device_name),
            address = COALESCE(?, address),
            parent_node_id = COALESCE(?, parent_node_id),
            time_for_recall = COALESCE(?, time_for_recall),
            timeout = COALESCE(?, timeout),
            retry_count = COALESCE(?, retry_count),
            is_active = COALESCE(?, is_active),
            read_by_group = COALESCE(?, read_by_group),
            description = COALESCE(?, description)
         WHERE id = ?"
    )
        .bind(device.device_name.clone())
        .bind(device.address)
        .bind(device.parent_node_id)
        .bind(device.time_for_recall)
        .bind(device.timeout)
        .bind(device.retry_count)
        .bind(device.is_active)
        .bind(device.read_by_group)
        .bind(device.description.clone())
        .bind(device.id)
        .execute(pool)
        .await
        .map_err(|e| {
            let msg = format!("Помилка оновлення пристрою:\n {}", e);
            printers::err(msg.clone());
            msg
        })?;

    let msg = format!("Оновлено пристрій по запиту: {:?}", device);
    printers::event(msg);

    Ok(())
}