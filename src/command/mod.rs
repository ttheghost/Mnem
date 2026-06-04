pub mod echo;
pub mod ping;

use crate::command::echo::Echo;
use crate::command::ping::Ping;
use crate::resp::RespValue;
use crate::server::Client;
use std::io::Error;

#[derive(Debug)]
pub struct Unknown {}

impl Unknown {
    fn new(cmd_kwd: String, args: &Vec<String>) -> Unknown {
        Unknown {}
    }
}

#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Unknown(Unknown),
}

impl Command {
    pub fn from_resp(resp_value: RespValue) -> Result<Command, Box<dyn std::error::Error>> {
        // all commands must be a RespValue::Array
        if let RespValue::Array(elements) = resp_value {
            // Array elements are Bulk String (RespValue::String)
            let mut str_lst = Vec::with_capacity(elements.len());
            for element in elements {
                if let RespValue::String(str) = element {
                    str_lst.push(str.to_owned());
                } else {
                    return Err(
                        Error::new(std::io::ErrorKind::InvalidInput, "invalid command").into(),
                    );
                }
            }
            let command_keyword = match str_lst.get(0) {
                None => {
                    return Err(
                        Error::new(std::io::ErrorKind::InvalidInput, "invalid command").into(),
                    );
                }
                Some(cmd_kwd) => cmd_kwd.to_lowercase(),
            };

            let args = str_lst.get(1..).unwrap_or(&[]).to_owned();

            let cmd = match command_keyword.as_str() {
                "ping" => Command::Ping(Ping::try_new(&args)?),
                "echo" => Command::Echo(Echo::try_new(&args)?),
                _ => {
                    return Ok(Command::Unknown(Unknown::new(command_keyword, &args)));
                }
            };

            Ok(cmd)
        } else {
            Err(Error::new(std::io::ErrorKind::InvalidInput, "invalid command").into())
        }
    }

    pub async fn execute(&self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Command::Ping(p) => p.execute(client).await,
            Command::Echo(e) => e.execute(client).await,
            Command::Unknown(_) => Ok(()),
        }
    }
}
