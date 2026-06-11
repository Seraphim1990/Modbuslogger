use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashedValue {
    pub tag: String,
    pub timestamp: i64,
    pub value: serde_json::Value,
}


#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ClientMessage {
    Update { update: Vec<HashedValue> },
    Get { get: Vec<String> },
    GetAll { get_all: bool },
}


#[derive(Default)]
pub struct ValueVault {
    pub values: Mutex<HashMap<String, HashedValue>>,
}

impl ValueVault {
    pub fn update_values(&self, new_values: Vec<HashedValue>) {
        let mut store = self.values.lock().unwrap();
        for v in new_values {
            store.insert(v.tag.clone(), v);
        }
    }

    pub fn get_many(&self, tags: &[String]) -> Vec<HashedValue> {
        let store = self.values.lock().unwrap();
        tags.iter()
            .filter_map(|tag| store.get(tag).cloned())
            .collect()
    }

    pub fn get_all(&self) -> Vec<HashedValue> {
        self.values.lock().unwrap().values().cloned().collect()
    }
}
