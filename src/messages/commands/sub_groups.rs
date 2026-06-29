use crate::db::schemas::user_subgroups::{UserSubGroupCreate, UserSubGroupDelete, UserSubGroupUpdate};

pub enum SubGroupCommand {
    Create(UserSubGroupCreate),
    Delete(UserSubGroupDelete),
    Update(UserSubGroupUpdate)
}