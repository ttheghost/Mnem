use crate::resp::{Resp, RespValue};
use crate::server::Client;

/// Ping command
/// https://redis.io/docs/latest/commands/ping/
#[derive(Debug)]
pub struct Ping {
    message: Option<String>,
}

impl Ping {
    pub fn try_new(args: &Vec<String>) -> Result<Ping, std::io::Error> {
        Ok(if args.len() == 1 {
            Ping {
                message: Some(args[0].clone()),
            }
        } else {
            Ping { message: None }
        })
    }

    pub async fn execute(
        &self,
        client: &mut Client,
    ) -> Result<RespValue, Box<dyn std::error::Error>> {
        Ok(if let Some(message) = &self.message {
            RespValue::String(message.clone())
        } else {
            RespValue::SimpleString("PONG".into())
        })
    }
}
