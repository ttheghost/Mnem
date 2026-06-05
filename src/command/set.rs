use crate::resp::{Resp, RespValue};
use crate::server::Client;
use crate::value::Value;

#[derive(Debug)]
pub struct Set {
    key: String,
    value: String,
}

impl Set {
    pub fn try_new(args: &Vec<String>) -> Result<Set, std::io::Error> {
        if args.len() != 2 {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: set <key> <value>",
            ))
        } else {
            let key = args[0].clone();
            let value = args[1].clone();
            Ok(Self { key, value })
        }
    }

    pub async fn execute(&self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut db = client.db.lock().unwrap();
            db.set(self.key.clone(), Value::String(self.value.clone()));
        }
        Resp::encode(RespValue::SimpleString("OK".into()), &mut client.socket).await?;
        Ok(())
    }
}
