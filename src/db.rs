use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<DbInner>>;

#[derive(Debug)]
pub struct DbInner {
    store: HashMap<String, Value>,
}
