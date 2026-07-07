use std::ops::Deref;
use sqlx::{FromRow, MySql, Pool};
use crate::messages::commands::{
    command::{Command, CommandType},
    sub_groups::SubGroupCommand
};
use crate::logger::printers;
use crate::db::schemas::user_subgroups::*;
use crate::messages::requests::sub_group_request::SubGroupsRequest;

pub fn user_sub_group_command(pool: &Pool<MySql>, command: Command){
    if let CommandType::SubGroupCommand(group) = command.cmd {
        let pool = pool.clone();
        match group.as_ref() {
            SubGroupCommand::Create(_) => {
                let group = group.clone();
                tokio::spawn(async move {
                   let tx = command.request_channel;
                    if let SubGroupCommand::Create(group) = group.deref() {
                        let res = create_subgroup(&pool, group).await;
                        if tx.send(res).is_err() {
                            printers::err("Помилка відправки калбеку SubGroupCommand::Create".to_string())
                        }
                    }
                });
            },
            SubGroupCommand::Update(_) => {
                let group = group.clone();
                tokio::spawn(async move {
                    let tx = command.request_channel;
                    if let SubGroupCommand::Update(group) = group.deref() {
                        let res = update_subgroup(&pool, group).await;
                        if tx.send(res).is_err() {
                            printers::err("Помилка відправки калбеку SubGroupCommand::Update".to_string())
                        }
                    }
                });
            },
            SubGroupCommand::Delete(_) => {
                let group = group.clone();
                tokio::spawn(async move {
                    let tx = command.request_channel;
                    if let SubGroupCommand::Delete(group) = group.deref() {
                        let res = delete_subgroup(&pool, group.id).await;
                        if tx.send(res).is_err() {
                            printers::err("Помилка відправки калбеку SubGroupCommand::Delete".to_string())
                        }
                    }
                });
            }
        }
    }
}

pub fn user_sub_group_get(pool: &Pool<MySql>, request: SubGroupsRequest){
    let pool = pool.clone();
    match request {
        SubGroupsRequest::GetById(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_by_id(&pool, request.id).await;
                if tx.send(res).is_err() {
                    printers::err("Помилка відправки калбеку SubGroupsRequest::GetById".to_string())
                }
            });
        },
        SubGroupsRequest::GetAll(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_all(&pool).await;
                if tx.send(res).is_err() {
                    printers::err("Помилка відправки калбеку SubGroupsRequest::GetAll".to_string())
                }
            });
        },
        SubGroupsRequest::GetByGroupId(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = get_by_group_id(&pool, request.group_id).await;
                if tx.send(res).is_err() {
                    printers::err("Помилка відправки калбеку SubGroupsRequest::GetByGroupId".to_string())
                }
            });
        },
        SubGroupsRequest::GetAssign(request) => {
            tokio::spawn(async move {
                let tx = request.request_channel;
                let res = ui_get_subgroups_assign(&pool, &request.groups_ids).await;
                if tx.send(res).is_err() {
                    printers::err("Помилка відправки калбеку SubGroupsRequest::GetAssign".to_string())
                }
            });
        }
    }
}

async fn get_by_id(pool: &Pool<MySql>, id: i32) -> Result<Option<UserSubGroupRead>, ()> {
    let subgroup = sqlx::query_as::<_, UserSubGroupRead>("SELECT id, group_id, subgroup_name, description FROM user_subgroups WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e|{
            printers::err(format!("Помилка читання бази даних: {}", e));
        })?;
    Ok(subgroup)
}

async fn get_all(pool: &Pool<MySql>) -> Result<Vec<UserSubGroupRead>, ()> {
    let subgroups = sqlx::query_as::<_, UserSubGroupRead>("SELECT id, group_id, subgroup_name, description FROM user_subgroups")
        .fetch_all(pool)
        .await
        .map_err(|e|{
            printers::err(format!("Помилка читання бази даних: {}", e));
        })?;
    Ok(subgroups)
}

pub async fn get_by_group_id(pool: &Pool<MySql>, group_id: i32) -> Result<Vec<UserSubGroupRead>, ()> {
    let subgroups = sqlx::query_as::<_, UserSubGroupRead>(
        "SELECT id, group_id, subgroup_name, description
         FROM user_subgroups
         WHERE group_id = ?"
    )
        .bind(group_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            printers::err(format!("Помилка читання бази даних: {}", e));
        })?;

    Ok(subgroups)
}

async fn create_subgroup(pool: &Pool<MySql>, subgroup: &UserSubGroupCreate) -> Result<(), String> {
    sqlx::query("INSERT INTO user_subgroups (group_id, subgroup_name, description) VALUES (?, ?, ?)")
        .bind(&subgroup.group_id)
        .bind(&subgroup.subgroup_name)
        .bind(&subgroup.description)
        .execute(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка створення підгрупи: {}", e);
            printers::err(msg.clone());
            msg
        })?;
    Ok(())
}

async fn update_subgroup(pool: &Pool<MySql>, subgroup: &UserSubGroupUpdate) -> Result<(), String> {
    sqlx::query("UPDATE user_subgroups SET group_id = COALESCE(?, group_id), subgroup_name = COALESCE(?, subgroup_name), description = COALESCE(?, description) WHERE id = ?")
        .bind(&subgroup.group_id)
        .bind(&subgroup.subgroup_name)
        .bind(&subgroup.description)
        .bind(subgroup.id)
        .execute(pool)
        .await.map_err(|e|{
            let msg = format!("Помилка оновлення підгрупи: {}", e);
            printers::err(msg.clone());
            msg
        })?;
    Ok(())
}

async fn delete_subgroup(pool: &Pool<MySql>, id: i32) -> Result<(), String> {
    sqlx::query("DELETE FROM user_subgroups WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e|{
            let msg = format!("Помилка видалення підгрупи: {}", e);
            printers::err(msg.clone());
            msg
        })?;
    Ok(())
}

#[derive(FromRow)]
struct ValUnitId { value_unit_id: i32 }
// UI
async fn ui_get_subgroups_assign(pool: &Pool<MySql>, subgroup_ids: &Vec<i32>) -> Result<Vec<GetSubgroupAssign>, ()> {
    let mut assignments = Vec::with_capacity(subgroup_ids.len());
    for subgroup in subgroup_ids.iter() {
        let val_ids = sqlx::query_as::<_, ValUnitId>("SELECT value_unit_id FROM subgroup_values WHERE subgroup_id = ?")
            .bind(subgroup)
            .fetch_all(pool)
            .await
            .map_err(|e|{
                printers::err(format!("Помилка отримання значень для субгруп: {}", e));
            })?;
        assignments.push(
            GetSubgroupAssign{
                    id: *subgroup,
                    values_id: val_ids.iter().map(|val_id|val_id.value_unit_id).collect()
            }
        );
    }
    Ok(assignments)
}