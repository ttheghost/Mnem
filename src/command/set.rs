use crate::resp::RespValue;
use crate::server::Client;
use crate::value::Value;
use std::ops::Add;
use std::time::Instant;

#[derive(Debug)]
pub struct Set {
    key: String,
    value: String,
    expire: Option<Instant>,
}

impl Set {
    pub fn try_new(args: &Vec<String>) -> Result<Set, std::io::Error> {
        if args.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: set <key> <value>",
            ));
        }
        let key = args[0].clone();
        let value = args[1].clone();
        let mut expire = None;
        if let Some(s) = args.get(2) {
            match s.to_uppercase().as_str() {
                "EX" => {
                    if let Some(s) = args.get(3) {
                        if let Ok(n) = s.parse::<u64>() {
                            let now = Instant::now();
                            let expire_time = now.add(std::time::Duration::from_secs(n));
                            expire = Some(expire_time);
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "Expire must be a number",
                            ));
                        }
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Expire must be a number",
                        ));
                    }
                }
                "PX" => {
                    if let Some(s) = args.get(3) {
                        if let Ok(n) = s.parse::<u64>() {
                            let now = Instant::now();
                            let expire_time = now.add(std::time::Duration::from_millis(n));
                            expire = Some(expire_time);
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "Expire must be a number",
                            ));
                        }
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Expire must be a number",
                        ));
                    }
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Usage: set <key> <value> [EX|PX] <seconds|milliseconds>",
                    ));
                }
            }
        }
        Ok(Self { key, value, expire })
    }

    pub async fn execute(
        &self,
        client: &mut Client,
    ) -> Result<RespValue, Box<dyn std::error::Error>> {
        {
            let mut db = client.db.lock().unwrap();
            db.set(
                self.key.clone(),
                Value::String(self.value.clone()),
                self.expire,
            );
        }
        Ok(RespValue::SimpleString("OK".into()))
    }
}
