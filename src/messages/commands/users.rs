use crate::db::schemas::users::{UserCreate, UserUpdate, UserDelete};

pub enum UserCommand {
    Create(UserCreate),
    Delete(UserDelete),
    Update(UserUpdate)
}