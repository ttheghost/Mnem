use crate::resp::{Resp, RespValue};
use crate::server::Client;
use crate::value::Value;

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn try_new(args: &Vec<String>) -> Result<Get, std::io::Error> {
        if args.len() != 1 {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: get <key>",
            ))
        } else {
            let key = args[0].clone();
            Ok(Self { key })
        }
    }

    pub async fn execute(
        &self,
        client: &mut Client,
    ) -> Result<RespValue, Box<dyn std::error::Error>> {
        let val = {
            let db = client.db.lock().unwrap();
            db.get(self.key.as_str()).cloned()
        };
        Ok(match val {
            Some(val) => match val {
                Value::Int(i) => RespValue::Integer(i),
                Value::String(s) => RespValue::String(s),
            },
            None => RespValue::NullString,
        })
    }
}
