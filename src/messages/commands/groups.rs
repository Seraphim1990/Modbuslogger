use crate::db::schemas::user_groups::{UserGroupCreate, UserGroupDelete, UserGroupUpdate};

pub enum GroupCommand {
    Create(UserGroupCreate),
    Delete(UserGroupDelete),
    Update(UserGroupUpdate),
}