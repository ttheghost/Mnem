use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<DbInner>>;

#[derive(Debug)]
pub struct DbInner {
    store: HashMap<String, Value>,
}

impl DbInner {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }
}

pub fn new_db() -> Db {
    Db::new(Mutex::new(DbInner::new()))
}
