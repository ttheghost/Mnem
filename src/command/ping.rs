use crate::resp::{Resp, RespValue};
use crate::server::Client;

/// Ping command
/// https://redis.io/docs/latest/commands/ping/
#[derive(Debug)]
pub struct Ping {
    message: Option<String>,
}

impl Ping {
    pub fn new(args: &Vec<String>) -> Ping {
        if args.len() == 1 {
            Ping { message: Some(args[0].clone()) }
        } else {
            Ping { message: None }
        }
    }

    pub async fn execute(&self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(message) = &self.message {
            Resp::encode(RespValue::String(message.clone()), &mut client.socket).await?;
        } else {
            Resp::encode(RespValue::SimpleString("PONG".into()), &mut client.socket).await?;
        }
        Ok(())
    }
}