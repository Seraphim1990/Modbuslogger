// Для прив'язки цехів до юзера
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AssignGroupsCommand {
    pub user_id: i32,
    pub group_ids: Vec<i32>,
}

// Для прив'язки регістрів до установки (підгрупи)
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AssignValuesCommand {
    pub subgroup_id: i32,
    pub value_unit_ids: Vec<i32>,
}

pub enum  AssignGroupsAndValuesCommand {
    Groups(AssignGroupsCommand),
    Values(AssignValuesCommand),
}