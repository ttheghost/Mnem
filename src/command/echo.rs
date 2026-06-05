use crate::resp::{Resp, RespValue};
use crate::server::Client;

/// Echo command
/// https://redis.io/docs/latest/commands/echo/
#[derive(Debug)]
pub struct Echo {
    pub message: String,
}

impl Echo {
    pub fn try_new(args: &Vec<String>) -> Result<Echo, std::io::Error> {
        if args.len() != 1 {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Usage: echo <command>",
            ))
        } else {
            Ok(Echo {
                message: args[0].clone(),
            })
        }
    }

    pub async fn execute(
        &self,
        client: &mut Client,
    ) -> Result<RespValue, Box<dyn std::error::Error>> {
        Ok(RespValue::String(self.message.clone()))
    }
}
