use crate::db::schemas::value_unit::{ValueUpdate, ValueCreate, ValueDelete};

pub enum ValueCommand {
    Create(ValueCreate),
    Delete(ValueDelete),
    Update(ValueUpdate),
}