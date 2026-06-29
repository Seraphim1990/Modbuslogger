use std::ops::Deref;
use sqlx::{MySql, Pool};
use crate::messages::commands::{
    command::{Command, CommandType},
    groups::GroupCommand
};
use crate::logger::printers;
use crate::db::schemas::user_groups::{UserGroupCreate, UserGroupRead, UserGroupUpdate, UserGroupDelete, UiUserGroupRead, UiUserSubGroupsRead};
use crate::messages::requests::group_request::GroupRequest;

pub fn group_command(pool: &Pool<MySql>, command: Command){
    let pool = pool.clone();
    if let CommandType::GroupCommand(group) = command.cmd {
        let tx = command.request_channel;
        match group.as_ref() {
            GroupCommand::Create(_) => {
                let group = group.clone();
                tokio::spawn(async move {
                   if let GroupCommand::Create(group) = group.deref() {
                       let res = create_group(&pool, group).await;
                       if tx.send(res).is_err() {
                           printers::err("Помилка відправки калбеку GroupCommand::Create".to_string())
                       }
                   }
                });
            },
            GroupCommand::Update(_) => {
                let group = group.clone();
                tokio::spawn(async move {
                    if let GroupCommand::Update(group) = group.deref() {
                        let res = update_group(&pool, group).await;
                        if tx.send(res).is_err() {
                            printers::err("Помилка відправки калбеку GroupCommand::Update".to_string())
                        }
                    }
                });
            },
            GroupCommand::Delete(_) => {
                let group = group.clone();
                tokio::spawn(async move {
                    if let GroupCommand::Delete(group) = group.deref() {
                        let res = delete_group(&pool, group.id).await;
                        if tx.send(res).is_err() {
                            printers::err("Помилка відправки калбеку GroupCommand::Delete".to_string())
                        }
                    }
                });
            },
        }
    }
}

pub fn groups_get(pool: &Pool<MySql>, request: GroupRequest){
    let pool = pool.clone();
    match request {
        GroupRequest::GetById(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_group_by_id(&pool, request.group_id).await;
                if tx.send(res).is_err(){
                    printers::err("Помилка відправки калбеку GroupRequest::GetById".to_string())
                }
            });
        },
        GroupRequest::GetAll(request) => {
            tokio::spawn(async move {
               let tx = request.request_channel;
                let res = get_all_groups(&pool).await;
                if tx.send(res).is_err() {
                    printers::err("Помилка відправки калбеку GroupRequest::GetAll".to_string())
                }
            });
        },
        GroupRequest::GetByUserId(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_group_by_user_id(&pool, request.user_id).await;
                if tx.send(res).is_err() {
                    printers::err("Помилка відправки калбеку GroupRequest::GetByUserId".to_string())
                }
            });
        },
        GroupRequest::GetForUi(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = ui_get_group(&pool, request.id).await;
                if tx.send(res).is_err() {
                    printers::err("Помилка відправки калбеку GroupRequest::GetForUi".to_string())
                }
            });
        }
    }
}

async fn get_group_by_id(pool: &Pool<MySql>, id: i32) -> Result<Option<UserGroupRead>, ()> {
    let group = sqlx::query_as::<_, UserGroupRead>("SELECT id, group_name, description FROM user_groups WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e|{
            printers::err(format!("Помилка читання бази даних: {}", e));
        })?;

    Ok(group)
}
async fn get_group_by_user_id(pool: &Pool<MySql>, user_id: i32) -> Result<Vec<UserGroupRead>, ()> {
    let groups = sqlx::query_as::<_, UserGroupRead>("SELECT g.id, g.group_name, g.description
                                                    FROM user_groups g
                                                    INNER JOIN user_group_access uga ON g.id = uga.group_id
                                                    WHERE uga.user_id = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e|{
            printers::err(format!("Помилка читання бази даних: {}", e));
        })?;
    Ok(groups)
}

async fn get_all_groups(pool: &Pool<MySql>) -> Result<Vec<UserGroupRead>, ()> {
    let groups = sqlx::query_as::<_, UserGroupRead>("SELECT id, group_name, description FROM user_groups")
        .fetch_all(pool)
        .await
        .map_err(|e|{
            printers::err(format!("Помилка читання бази даних: {}", e));
        })?;
    Ok(groups)
}

async fn create_group(pool: &Pool<MySql>, group: &UserGroupCreate) -> Result<(), String> {
    sqlx::query("INSERT INTO user_groups (group_name, description) VALUES (?, ?)")
        .bind(&group.group_name)
        .bind(&group.description)
        .execute(pool)
        .await
        .map_err(|e| {
            let msg = format!("Помилка створення групи: {}", e);
            printers::err(msg.clone());
            msg
        })?;
    Ok(())
}

async fn update_group(pool: &Pool<MySql>, group: &UserGroupUpdate) -> Result<(), String> {
    sqlx::query("UPDATE user_groups SET group_name = COALESCE(?, group_name), description = COALESCE(?, description) WHERE id = ?")
        .bind(&group.group_name)
        .bind(&group.description)
        .bind(group.id)
        .execute(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка оновлення групи: {}", e);
            printers::err(msg.clone());
            msg
        })?;
    Ok(())
}

async fn delete_group(pool: &Pool<MySql>, group_id: i32) -> Result<(), String> {
    sqlx::query("DELETE FROM user_groups WHERE id = ?")
        .bind(group_id)
        .execute(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка Видалення групи: {}", e);
            printers::err(msg.clone());
            msg
        })?;
    Ok(())
}

// UI block

use crate::db::schemas::value_unit::ValueRead;
async fn ui_get_group(pool: &Pool<MySql>, id: i32) -> Result<Option<UiUserGroupRead>, ()> {

    let subgroups = crate::db::worker::user_sub_group_workers::get_by_group_id(pool, id).await?; // костиль, потом поправлю!!

    if subgroups.is_empty() { return Ok(None) }

    let mut res = UiUserGroupRead { id, sub_groups: Vec::new() };

    for subgroup in subgroups.into_iter() {
        let grouped_values = sqlx::query_as::<_, ValueRead>
            ("SELECT vu.*
            FROM `value_units` vu
            INNER JOIN `subgroup_values` sgv ON vu.`id` = sgv.`value_unit_id`
            WHERE sgv.`subgroup_id` = ?;")
            .bind(subgroup.id)
            .fetch_all(pool)
            .await
            .map_err(|e|{
                printers::err(format!("Помилка отримання значень для підгрупЖ {}", e));
            })?;

        res.sub_groups.push(
            UiUserSubGroupsRead {
                group_state: subgroup,
                values: grouped_values,
            }
        )
    }
    Ok(Some(res))
}
