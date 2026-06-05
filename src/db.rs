use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub type Db = Arc<Mutex<DbInner>>;

#[derive(Debug)]
pub struct DbInner {
    store: HashMap<String, Value>,
    expires: HashMap<String, SystemTime>,
}

impl DbInner {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            expires: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: Value, expires: Option<SystemTime>) {
        self.store.insert(key.clone(), value);
        if let Some(expires) = expires {
            self.expires.insert(key, expires);
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&Value> {
        if let Some(expires) = self.expires.get(key) {
            let now = SystemTime::now();
            if now > *expires {
                self.store.remove(key);
                self.expires.remove(key);
                return None;
            }
        }
        self.store.get(key)
    }
}

pub fn new_db() -> Db {
    Db::new(Mutex::new(DbInner::new()))
}
