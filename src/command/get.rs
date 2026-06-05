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

    pub async fn execute(&self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        let val = {
            let db = client.db.lock().unwrap();
            db.get(self.key.as_str()).cloned()
        };
        match val {
            Some(val) => match val {
                Value::Int(i) => Resp::encode(RespValue::Integer(i), &mut client.socket).await?,
                Value::String(s) => Resp::encode(RespValue::String(s), &mut client.socket).await?,
            },
            None => Resp::encode(RespValue::NullString, &mut client.socket).await?,
        }
        Ok(())
    }
}
