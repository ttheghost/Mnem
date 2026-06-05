use crate::command::get::Get;
use crate::resp::RespValue;
use crate::server::Client;
use crate::value::Value;

#[derive(Debug)]
pub struct Type {
    key: String,
}

impl Type {
    pub fn try_new(args: &Vec<String>) -> Result<Type, std::io::Error> {
        if args.len() != 1 {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: type <key>",
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
            let mut db = client.db.lock().unwrap();
            db.get(self.key.as_str()).cloned()
        };
        Ok(match val {
            Some(val) => match val {
                Value::Int(_) => RespValue::SimpleString("string".into()),
                Value::String(_) => RespValue::SimpleString("string".into()),
            },
            None => RespValue::SimpleString("none".into()),
        })
    }
}